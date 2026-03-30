use crate::addr::flush_response::AddrFlushResponse;
use neli::consts::nl::NlmF;
use neli::consts::rtnl::{RtAddrFamily, RtScope, Rtm};
use neli::nl::NlPayload;
use neli::router::synchronous::NlRouterReceiverHandle;
use neli::rtnl::{Ifaddrmsg, IfaddrmsgBuilder};
use nix::net::if_::if_nametoindex;

use crate::NetlinkRouteHandle;
use crate::error::Error;

pub struct AddrFlushRequest {
    if_index: u32,
}

impl AddrFlushRequest {
    pub fn for_ifname(ifname: &str) -> Result<Self, Error> {
        let if_index = if_nametoindex(ifname).map_err(|e| Error::InterfaceLookup {
            ifname: ifname.to_owned(),
            source: e,
        })?;
        Ok(AddrFlushRequest { if_index })
    }

    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<AddrFlushResponse, Error> {
        let mut addresses_flushed: usize = 0;

        let ifaddrmsg = IfaddrmsgBuilder::default()
            .ifa_family(RtAddrFamily::Inet)
            .ifa_prefixlen(0)
            .ifa_scope(RtScope::Universe)
            .ifa_index(self.if_index)
            .build()?;

        let recv: NlRouterReceiverHandle<Rtm, Ifaddrmsg> = h
            .rtnl
            .send(Rtm::Getaddr, NlmF::DUMP, NlPayload::Payload(ifaddrmsg))
            .map_err(|e| Error::Send(Box::new(e)))?;

        // Collect addresses to delete from this round.
        let mut to_delete: Vec<Ifaddrmsg> = vec![];

        for response in recv {
            let header = response.map_err(|e| Error::Receive(Box::new(e)))?;
            if let NlPayload::Payload(p) = header.nl_payload() {
                if header.nl_type() != &Rtm::Newaddr {
                    return Err(Error::UnexpectedNlType {
                        expected: format!("{:?}", Rtm::Newaddr),
                        actual: format!("{:?}", header.nl_type()),
                    });
                }
                //TODO: is there any point to this check?
                if *p.ifa_index() == self.if_index {
                    to_delete.push(p.clone());
                } else {
                    return Err(Error::InvalidDataInResponse {
                        reason: "received ifindex violating filter",
                    });
                }
            }
        }

        for addr_msg in to_delete {
            let recv: NlRouterReceiverHandle<Rtm, Ifaddrmsg> = h
                .rtnl
                .send(Rtm::Deladdr, NlmF::ACK, NlPayload::Payload(addr_msg))
                .map_err(|e| Error::Send(Box::new(e)))?;

            for response in recv {
                response.map_err(|e| Error::Receive(Box::new(e)))?;
            }

            addresses_flushed += 1;
        }

        Ok(AddrFlushResponse { addresses_flushed })
    }
}
