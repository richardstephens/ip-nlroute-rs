use crate::NetlinkRouteHandle;
use crate::error::Error;
use macaddr::MacAddr6;

use crate::util::mappers::interface::resolve_ifname;
#[cfg(all(target_os = "linux", feature = "netlink"))]
use neli::{
    Size, ToBytes,
    consts::nl::NlmF,
    consts::rtnl::{Ifla, IflaInfo, RtAddrFamily, Rtm},
    neli_enum,
    nl::NlPayload,
    router::synchronous::NlRouterReceiverHandle,
    rtnl::{Ifinfomsg, IfinfomsgBuilder, Rtattr, RtattrBuilder},
    types::{Buffer, RtBuffer},
};

#[derive(Default, Clone, Debug)]
pub struct BridgeOptions {
    stp: Option<bool>,
    forward_delay: Option<u32>,
    hello_time: Option<u32>,
    max_age: Option<u32>,
    ageing_time: Option<u32>,
    priority: Option<u16>,
    vlan_filtering: Option<bool>,
}

impl BridgeOptions {
    /// Create an empty set of options that changes nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable IEEE 802.1d Spanning Tree Protocol (`IFLA_BR_STP_STATE`).
    pub fn stp(mut self, on: bool) -> Self {
        self.stp = Some(on);
        self
    }

    /// Set the forward delay (`IFLA_BR_FORWARD_DELAY`).
    pub fn forward_delay(mut self, value: u32) -> Self {
        self.forward_delay = Some(value);
        self
    }

    /// Set the hello time (`IFLA_BR_HELLO_TIME`).
    pub fn hello_time(mut self, value: u32) -> Self {
        self.hello_time = Some(value);
        self
    }

    /// Set the max age (`IFLA_BR_MAX_AGE`).
    pub fn max_age(mut self, value: u32) -> Self {
        self.max_age = Some(value);
        self
    }

    /// Set the FDB ageing time (`IFLA_BR_AGEING_TIME`).
    pub fn ageing_time(mut self, value: u32) -> Self {
        self.ageing_time = Some(value);
        self
    }

    /// Set the bridge priority (`IFLA_BR_PRIORITY`); lower wins root election.
    pub fn priority(mut self, value: u16) -> Self {
        self.priority = Some(value);
        self
    }

    /// Enable or disable 802.1q VLAN filtering (`IFLA_BR_VLAN_FILTERING`).
    pub fn vlan_filtering(mut self, on: bool) -> Self {
        self.vlan_filtering = Some(on);
        self
    }

    /// Whether no option has been set (so there is nothing to send).
    pub fn is_empty(&self) -> bool {
        self.stp.is_none()
            && self.forward_delay.is_none()
            && self.hello_time.is_none()
            && self.max_age.is_none()
            && self.ageing_time.is_none()
            && self.priority.is_none()
            && self.vlan_filtering.is_none()
    }

    /// Encode the options as the nested `IFLA_INFO_DATA` attribute buffer.
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    fn to_info_data(&self) -> Result<RtBuffer<IflaBr, Buffer>, Error> {
        let mut data = RtBuffer::<IflaBr, Buffer>::new();
        if let Some(on) = self.stp {
            data.push(br_attr(IflaBr::StpState, on as u32)?);
        }
        if let Some(v) = self.forward_delay {
            data.push(br_attr(IflaBr::ForwardDelay, v)?);
        }
        if let Some(v) = self.hello_time {
            data.push(br_attr(IflaBr::HelloTime, v)?);
        }
        if let Some(v) = self.max_age {
            data.push(br_attr(IflaBr::MaxAge, v)?);
        }
        if let Some(v) = self.ageing_time {
            data.push(br_attr(IflaBr::AgeingTime, v)?);
        }
        if let Some(v) = self.priority {
            data.push(br_attr(IflaBr::Priority, v)?);
        }
        if let Some(on) = self.vlan_filtering {
            data.push(br_attr(IflaBr::VlanFiltering, on as u8)?);
        }
        Ok(data)
    }
}

#[derive(Debug)]
pub struct BridgeCreateResponse {
    /// Name the bridge was created with.
    pub name: String,
}

#[derive(Debug)]
pub struct BridgeSetResponse {
    /// Index of the bridge that was modified.
    pub if_index: u32,
}

pub struct BridgeCreateRequest {
    name: String,
    options: BridgeOptions,
    address: Option<[u8; 6]>,
}

impl BridgeCreateRequest {
    pub fn new(name: impl Into<String>) -> Self {
        BridgeCreateRequest {
            name: name.into(),
            options: BridgeOptions::new(),
            address: None,
        }
    }

    pub fn options(mut self, options: BridgeOptions) -> Self {
        self.options = options;
        self
    }

    pub fn address(mut self, mac: MacAddr6) -> Self {
        self.address = Some(mac.into_array());
        self
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<BridgeCreateResponse, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<BridgeCreateResponse, Error> {
        let mut attrs = RtBuffer::<Ifla, Buffer>::new();
        attrs.push(
            RtattrBuilder::default()
                .rta_type(Ifla::Ifname)
                .rta_payload(self.name.as_str())
                .build()?,
        );
        if let Some(address) = self.address {
            attrs.push(
                RtattrBuilder::default()
                    .rta_type(Ifla::Address)
                    .rta_payload(&address[..])
                    .build()?,
            );
        }
        attrs.push(build_linkinfo(&self.options)?);

        let ifinfomsg = IfinfomsgBuilder::default()
            .ifi_family(RtAddrFamily::Unspecified)
            .rtattrs(attrs)
            .build()?;

        // CREATE | EXCL makes this fail if a link with the name already exists,
        // matching `ip link add`.
        let recv: NlRouterReceiverHandle<Rtm, Ifinfomsg> = h
            .rtnl
            .send(
                Rtm::Newlink,
                NlmF::CREATE | NlmF::EXCL | NlmF::ACK,
                NlPayload::Payload(ifinfomsg),
            )
            .map_err(|e| Error::Send(Box::new(e)))?;

        for response in recv {
            response.map_err(|e| Error::Receive(Box::new(e)))?;
        }

        Ok(BridgeCreateResponse {
            name: self.name.clone(),
        })
    }
}

/// Request to update the options of an existing bridge.
pub struct BridgeSetRequest {
    if_index: u32,
    options: BridgeOptions,
}

impl BridgeSetRequest {
    /// Build a request targeting the bridge with the given name.
    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn for_ifname(_ifname: &str) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    /// Build a request targeting the bridge with the given name.
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn for_ifname(ifname: &str) -> Result<Self, Error> {
        Ok(Self::for_index(resolve_ifname(ifname)?))
    }

    /// Build a request targeting the bridge with the given index.
    pub fn for_index(if_index: u32) -> Self {
        BridgeSetRequest {
            if_index,
            options: BridgeOptions::new(),
        }
    }

    /// Apply the given options to the bridge.
    pub fn options(mut self, options: BridgeOptions) -> Self {
        self.options = options;
        self
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<BridgeSetResponse, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<BridgeSetResponse, Error> {
        let mut attrs = RtBuffer::<Ifla, Buffer>::new();
        attrs.push(build_linkinfo(&self.options)?);

        let ifinfomsg = IfinfomsgBuilder::default()
            .ifi_family(RtAddrFamily::Unspecified)
            .ifi_index(self.if_index as i32)
            .rtattrs(attrs)
            .build()?;

        let recv: NlRouterReceiverHandle<Rtm, Ifinfomsg> = h
            .rtnl
            .send(Rtm::Newlink, NlmF::ACK, NlPayload::Payload(ifinfomsg))
            .map_err(|e| Error::Send(Box::new(e)))?;

        for response in recv {
            response.map_err(|e| Error::Receive(Box::new(e)))?;
        }

        Ok(BridgeSetResponse {
            if_index: self.if_index,
        })
    }
}

#[cfg(all(target_os = "linux", feature = "netlink"))]
fn build_linkinfo(options: &BridgeOptions) -> Result<Rtattr<Ifla, Buffer>, Error> {
    let mut info = RtBuffer::<IflaInfo, Buffer>::new();
    info.push(
        RtattrBuilder::default()
            .rta_type(IflaInfo::Kind)
            .rta_payload("bridge")
            .build()?,
    );
    if !options.is_empty() {
        info.push(
            RtattrBuilder::default()
                .rta_type(IflaInfo::Data)
                .rta_payload(options.to_info_data()?)
                .build()?,
        );
    }
    Ok(RtattrBuilder::default()
        .rta_type(Ifla::Linkinfo)
        .rta_payload(info)
        .build()?)
}

/// Build a single nested bridge attribute.
#[cfg(all(target_os = "linux", feature = "netlink"))]
fn br_attr<P: Size + ToBytes>(ty: IflaBr, payload: P) -> Result<Rtattr<IflaBr, Buffer>, Error> {
    Ok(RtattrBuilder::default()
        .rta_type(ty)
        .rta_payload(payload)
        .build()?)
}

/// Nested attribute types for `IFLA_INFO_DATA` of a bridge (`IFLA_BR_*`).
///
/// neli does not model these, so we define the subset we use. Values match
/// `linux/if_link.h`.
#[cfg(all(target_os = "linux", feature = "netlink"))]
#[neli_enum(serialized_type = "u16")]
pub(crate) enum IflaBr {
    Unspec = 0,
    ForwardDelay = 1,
    HelloTime = 2,
    MaxAge = 3,
    AgeingTime = 4,
    StpState = 5,
    Priority = 6,
    VlanFiltering = 7,
}
