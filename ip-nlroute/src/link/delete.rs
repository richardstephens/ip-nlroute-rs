use crate::NetlinkRouteHandle;
use crate::error::Error;
use crate::link::delete_response::LinkDeleteResponse;
use crate::util::mappers::interface::resolve_ifname;
#[cfg(all(target_os = "linux", feature = "netlink"))]
use neli::{
    consts::nl::NlmF,
    consts::rtnl::{RtAddrFamily, Rtm},
    nl::NlPayload,
    router::synchronous::NlRouterReceiverHandle,
    rtnl::{Ifinfomsg, IfinfomsgBuilder},
};

/// Request to delete a link (e.g. a bridge created with
/// [`BridgeCreateRequest`](crate::link::bridge::BridgeCreateRequest)).
pub struct LinkDeleteRequest {
    if_index: u32,
}

impl LinkDeleteRequest {
    /// Build a request targeting the interface with the given name.
    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn for_ifname(_ifname: &str) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    /// Build a request targeting the interface with the given name.
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn for_ifname(ifname: &str) -> Result<Self, Error> {
        Ok(Self::for_index(resolve_ifname(ifname)?))
    }

    /// Build a request targeting the interface with the given index.
    pub fn for_index(if_index: u32) -> Self {
        LinkDeleteRequest { if_index }
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<LinkDeleteResponse, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<LinkDeleteResponse, Error> {
        let ifinfomsg = IfinfomsgBuilder::default()
            .ifi_family(RtAddrFamily::Unspecified)
            .ifi_index(self.if_index as i32)
            .build()?;

        let recv: NlRouterReceiverHandle<Rtm, Ifinfomsg> = h
            .rtnl
            .send(Rtm::Dellink, NlmF::ACK, NlPayload::Payload(ifinfomsg))
            .map_err(|e| Error::Send(Box::new(e)))?;

        for response in recv {
            response.map_err(|e| Error::Receive(Box::new(e)))?;
        }

        Ok(LinkDeleteResponse {
            if_index: self.if_index,
        })
    }
}
