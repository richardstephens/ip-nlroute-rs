use crate::NetlinkRouteHandle;
use crate::error::Error;
use neli::router::synchronous::NlRouterReceiverHandle;
use neli::{
    attr::Attribute,
    consts::{
        nl::NlmF,
        rtnl::{Ifa, RtAddrFamily, RtScope, Rtm},
    },
    nl::{NlPayload, Nlmsghdr},
    rtnl::{Ifaddrmsg, IfaddrmsgBuilder},
};
use std::net::Ipv4Addr;

pub struct AddrGetRequest {}

impl AddrGetRequest {
    pub fn send(h: &mut NetlinkRouteHandle) -> Result<(), Error> {
        let ifaddrmsg = IfaddrmsgBuilder::default()
            .ifa_family(RtAddrFamily::Inet)
            .ifa_prefixlen(0)
            .ifa_scope(RtScope::Universe)
            .ifa_index(0)
            .build()?;
        let recv: NlRouterReceiverHandle<Rtm, Ifaddrmsg> = h
            .rtnl
            .send(Rtm::Getaddr, NlmF::ROOT, NlPayload::Payload(ifaddrmsg))
            .map_err(|_| Error::SendError)?;
        let mut addrs = Vec::<Ipv4Addr>::with_capacity(1);
        for response in recv {
            let header: Nlmsghdr<Rtm, Ifaddrmsg> =
                response.map_err(|_| Error::NlRouterMiscError)?;
            if let NlPayload::Payload(p) = header.nl_payload() {
                if header.nl_type() != &Rtm::Newaddr {
                    return Err(Error::UnexpectedNlType);
                }
                if p.ifa_scope() != &RtScope::Universe {
                    continue;
                }
                let idx = *p.ifa_index();
                let family = *p.ifa_family();

                println!("family: {:?} idx={idx}", family);

                for rtattr in p.rtattrs().iter() {
                    println!("{:#?}", rtattr);
                    if rtattr.rta_type() == &Ifa::Local {
                        addrs.push(Ipv4Addr::from(u32::from_be(
                            rtattr.get_payload_as::<u32>().map_err(|_| Error::DeError)?,
                        )));
                    }
                }
            }
        }

        println!("Local IPv4 addresses:");
        for addr in addrs {
            println!("{addr}");
        }

        Ok(())
    }
}
