use crate::NetlinkRouteHandle;
use crate::error::Error;
use crate::link::set_response::LinkSetResponse;
use crate::util::mappers::interface::resolve_ifname;
use macaddr::MacAddr6;
#[cfg(all(target_os = "linux", feature = "netlink"))]
use neli::{
    consts::nl::NlmF,
    consts::rtnl::{Iff, Ifla, RtAddrFamily, Rtm},
    nl::NlPayload,
    router::synchronous::NlRouterReceiverHandle,
    rtnl::{Ifinfomsg, IfinfomsgBuilder, RtattrBuilder},
    types::{Buffer, RtBuffer},
};

pub struct LinkSetRequest {
    if_index: u32,
    promisc: Option<bool>,
    // `None` leaves the master unchanged; `Some(0)` detaches the link from its
    // current master; `Some(idx)` enslaves it to the link with that index.
    master: Option<u32>,
    // `None` leaves the hardware address unchanged; `Some(mac)` pins it.
    address: Option<[u8; 6]>,
    // `None` leaves the MTU unchanged; `Some(mtu)` sets it.
    mtu: Option<u32>,
    // `None` leaves the admin state unchanged; `Some(true)` brings the link up,
    // `Some(false)` brings it down.
    up: Option<bool>,
}

impl LinkSetRequest {
    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn for_ifname(_ifname: &str) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn for_ifname(ifname: &str) -> Result<Self, Error> {
        Ok(Self::for_index(resolve_ifname(ifname)?))
    }

    /// Build a request targeting the interface with the given index.
    pub fn for_index(if_index: u32) -> Self {
        LinkSetRequest {
            if_index,
            promisc: None,
            master: None,
            address: None,
            mtu: None,
            up: None,
        }
    }

    /// Enable or disable promiscuous mode (`IFF_PROMISC`) on the link.
    pub fn promisc(mut self, on: bool) -> Self {
        self.promisc = Some(on);
        self
    }

    pub fn master(mut self, master_if_index: u32) -> Self {
        self.master = Some(master_if_index);
        self
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn master_ifname(self, _master: &str) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn master_ifname(self, master: &str) -> Result<Self, Error> {
        Ok(self.master(resolve_ifname(master)?))
    }

    /// Detach the link from its current master, e.g. to remove it from a
    /// bridge.
    pub fn nomaster(mut self) -> Self {
        self.master = Some(0);
        self
    }

    pub fn address(mut self, mac: MacAddr6) -> Self {
        self.address = Some(mac.into_array());
        self
    }

    pub fn mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    pub fn up(mut self, up: bool) -> Self {
        self.up = Some(up);
        self
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<LinkSetResponse, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<LinkSetResponse, Error> {
        // Translate the requested flag changes into the kernel's flags/change
        // mask pair: `change` selects which bits the kernel should honour and
        // `flags` carries the desired value of those bits.
        let mut flags = Iff::empty();
        let mut change = Iff::empty();
        if let Some(on) = self.promisc {
            change |= Iff::PROMISC;
            if on {
                flags |= Iff::PROMISC;
            }
        }
        if let Some(on) = self.up {
            change |= Iff::UP;
            if on {
                flags |= Iff::UP;
            }
        }

        let mut attrs = RtBuffer::<Ifla, Buffer>::new();
        if let Some(master) = self.master {
            attrs.push(
                RtattrBuilder::default()
                    .rta_type(Ifla::Master)
                    .rta_payload(master)
                    .build()?,
            );
        }
        if let Some(address) = self.address {
            attrs.push(
                RtattrBuilder::default()
                    .rta_type(Ifla::Address)
                    .rta_payload(&address[..])
                    .build()?,
            );
        }
        if let Some(mtu) = self.mtu {
            attrs.push(
                RtattrBuilder::default()
                    .rta_type(Ifla::Mtu)
                    .rta_payload(mtu)
                    .build()?,
            );
        }

        let ifinfomsg = IfinfomsgBuilder::default()
            .ifi_family(RtAddrFamily::Unspecified)
            .ifi_index(self.if_index as i32)
            .ifi_flags(flags)
            .ifi_change(change)
            .rtattrs(attrs)
            .build()?;

        let recv: NlRouterReceiverHandle<Rtm, Ifinfomsg> = h
            .rtnl
            .send(Rtm::Newlink, NlmF::ACK, NlPayload::Payload(ifinfomsg))
            .map_err(|e| Error::Send(Box::new(e)))?;

        // Drain the ACK so any error reported by the kernel surfaces here.
        for response in recv {
            response.map_err(|e| Error::Receive(Box::new(e)))?;
        }

        Ok(LinkSetResponse {
            if_index: self.if_index,
        })
    }
}
