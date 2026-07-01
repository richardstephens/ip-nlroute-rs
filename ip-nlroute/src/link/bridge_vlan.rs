//! Per-port (and per-bridge) VLAN membership.
//!
//! These mirror `bridge vlan add` / `bridge vlan del`: they program a VLAN ID
//! onto a bridge port (or onto the bridge itself with [`self_link`]). A VLAN
//! marked `pvid` + `untagged` turns the port into an "access port" for that
//! VLAN. VLAN filtering must be enabled on the bridge
//! ([`BridgeOptions::vlan_filtering`](crate::link::bridge::BridgeOptions::vlan_filtering))
//! for these to take effect.
//!
//! [`self_link`]: BridgeVlanAddRequest::self_link

use crate::NetlinkRouteHandle;
use crate::error::Error;

use crate::util::mappers::interface::resolve_ifname;
#[cfg(all(target_os = "linux", feature = "netlink"))]
use neli::{
    consts::nl::NlmF,
    consts::rtnl::{RtAddrFamily, Rtm},
    neli_enum,
    nl::NlPayload,
    router::synchronous::NlRouterReceiverHandle,
    rtnl::{Ifinfomsg, IfinfomsgBuilder, Rtattr, RtattrBuilder},
    types::{Buffer, RtBuffer},
};

#[cfg(all(target_os = "linux", feature = "netlink"))]
const AF_BRIDGE: u8 = 7;

// struct bridge_vlan_info flags (linux/if_bridge.h).
#[cfg(all(target_os = "linux", feature = "netlink"))]
const BRIDGE_VLAN_INFO_PVID: u16 = 1 << 1;
#[cfg(all(target_os = "linux", feature = "netlink"))]
const BRIDGE_VLAN_INFO_UNTAGGED: u16 = 1 << 2;

// IFLA_BRIDGE_FLAGS values (linux/if_bridge.h).
#[cfg(all(target_os = "linux", feature = "netlink"))]
const BRIDGE_FLAGS_SELF: u16 = 2;

/// Result of a VLAN membership change.
#[derive(Debug)]
pub struct BridgeVlanResponse {
    /// Index of the interface the VLAN was changed on.
    pub if_index: u32,
    /// The VLAN ID that was added or removed.
    pub vid: u16,
}

/// Request to add a VLAN to a bridge port (or the bridge itself).
pub struct BridgeVlanAddRequest {
    if_index: u32,
    vid: u16,
    pvid: bool,
    untagged: bool,
    self_link: bool,
}

impl BridgeVlanAddRequest {
    /// Target the interface with the given name and VLAN ID.
    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn for_ifname(_ifname: &str, _vid: u16) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    /// Target the interface with the given name and VLAN ID.
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn for_ifname(ifname: &str, vid: u16) -> Result<Self, Error> {
        Ok(Self::for_index(resolve_ifname(ifname)?, vid))
    }

    /// Target the interface with the given index and VLAN ID.
    pub fn for_index(if_index: u32, vid: u16) -> Self {
        BridgeVlanAddRequest {
            if_index,
            vid,
            pvid: false,
            untagged: false,
            self_link: false,
        }
    }

    /// Make this VLAN the port's PVID (untagged ingress is assigned to it).
    pub fn pvid(mut self, on: bool) -> Self {
        self.pvid = on;
        self
    }

    /// Egress frames for this VLAN untagged.
    pub fn untagged(mut self, on: bool) -> Self {
        self.untagged = on;
        self
    }

    /// Operate on the bridge device itself rather than treating the target as
    /// a member port (the `self` keyword in `bridge vlan`).
    pub fn self_link(mut self, on: bool) -> Self {
        self.self_link = on;
        self
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<BridgeVlanResponse, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<BridgeVlanResponse, Error> {
        let mut vinfo_flags = 0u16;
        if self.pvid {
            vinfo_flags |= BRIDGE_VLAN_INFO_PVID;
        }
        if self.untagged {
            vinfo_flags |= BRIDGE_VLAN_INFO_UNTAGGED;
        }
        send_vlan(
            h,
            Rtm::Setlink,
            self.if_index,
            self.vid,
            vinfo_flags,
            self.self_link,
        )?;
        Ok(BridgeVlanResponse {
            if_index: self.if_index,
            vid: self.vid,
        })
    }
}

/// Request to remove a VLAN from a bridge port (or the bridge itself).
pub struct BridgeVlanDelRequest {
    if_index: u32,
    vid: u16,
    self_link: bool,
}

impl BridgeVlanDelRequest {
    /// Target the interface with the given name and VLAN ID.
    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn for_ifname(_ifname: &str, _vid: u16) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    /// Target the interface with the given name and VLAN ID.
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn for_ifname(ifname: &str, vid: u16) -> Result<Self, Error> {
        Ok(Self::for_index(resolve_ifname(ifname)?, vid))
    }

    /// Target the interface with the given index and VLAN ID.
    pub fn for_index(if_index: u32, vid: u16) -> Self {
        BridgeVlanDelRequest {
            if_index,
            vid,
            self_link: false,
        }
    }

    /// Operate on the bridge device itself rather than treating the target as
    /// a member port (the `self` keyword in `bridge vlan`).
    pub fn self_link(mut self, on: bool) -> Self {
        self.self_link = on;
        self
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<BridgeVlanResponse, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<BridgeVlanResponse, Error> {
        send_vlan(h, Rtm::Dellink, self.if_index, self.vid, 0, self.self_link)?;
        Ok(BridgeVlanResponse {
            if_index: self.if_index,
            vid: self.vid,
        })
    }
}

/// Build and send an `AF_BRIDGE` VLAN message: an `IFLA_AF_SPEC` carrying an
/// `IFLA_BRIDGE_VLAN_INFO` (and `IFLA_BRIDGE_FLAGS` when operating on `self`).
#[cfg(all(target_os = "linux", feature = "netlink"))]
fn send_vlan(
    h: &mut NetlinkRouteHandle,
    cmd: Rtm,
    if_index: u32,
    vid: u16,
    vinfo_flags: u16,
    self_link: bool,
) -> Result<(), Error> {
    use neli::consts::rtnl::Ifla;

    let mut af_spec = RtBuffer::<IflaBridge, Buffer>::new();

    if self_link {
        af_spec.push(
            RtattrBuilder::default()
                .rta_type(IflaBridge::Flags)
                .rta_payload(BRIDGE_FLAGS_SELF)
                .build()?,
        );
    }

    // struct bridge_vlan_info { __u16 flags; __u16 vid; } in native byte order.
    let mut vinfo = [0u8; 4];
    vinfo[0..2].copy_from_slice(&vinfo_flags.to_ne_bytes());
    vinfo[2..4].copy_from_slice(&vid.to_ne_bytes());
    af_spec.push(
        RtattrBuilder::default()
            .rta_type(IflaBridge::VlanInfo)
            .rta_payload(&vinfo[..])
            .build()?,
    );

    let af_spec_attr: Rtattr<Ifla, Buffer> = RtattrBuilder::default()
        .rta_type(Ifla::AfSpec)
        .rta_payload(af_spec)
        .build()?;

    let mut attrs = RtBuffer::<Ifla, Buffer>::new();
    attrs.push(af_spec_attr);

    let ifinfomsg = IfinfomsgBuilder::default()
        .ifi_family(RtAddrFamily::from(AF_BRIDGE))
        .ifi_index(if_index as i32)
        .rtattrs(attrs)
        .build()?;

    let recv: NlRouterReceiverHandle<Rtm, Ifinfomsg> = h
        .rtnl
        .send(cmd, NlmF::ACK, NlPayload::Payload(ifinfomsg))
        .map_err(|e| Error::Send(Box::new(e)))?;

    for response in recv {
        response.map_err(|e| Error::Receive(Box::new(e)))?;
    }

    Ok(())
}

/// Nested attribute types for `IFLA_AF_SPEC` under `AF_BRIDGE` (`IFLA_BRIDGE_*`).
///
/// neli does not model these. Values match `linux/if_bridge.h`.
#[cfg(all(target_os = "linux", feature = "netlink"))]
#[neli_enum(serialized_type = "u16")]
pub(crate) enum IflaBridge {
    Flags = 0,
    Mode = 1,
    VlanInfo = 2,
}
