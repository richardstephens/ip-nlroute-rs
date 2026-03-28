use crate::NetlinkRouteHandle;
use crate::addr::{AddrGetInterface, AddrGetInterfaceAddressV4, AddrGetResponse, AddressFlags};
use crate::error::Error;
use crate::util::mappers::ip::{rtattr_to_ipv4, rtattr_to_string};
use neli::router::synchronous::NlRouterReceiverHandle;
use neli::{
    consts::{
        nl::NlmF,
        rtnl::{Ifa, RtAddrFamily, RtScope, Rtm},
    },
    nl::{NlPayload, Nlmsghdr},
    rtnl::{Ifaddrmsg, IfaddrmsgBuilder},
};
use nix::net::if_::{if_indextoname, if_nametoindex};
use std::collections::BTreeMap;

pub struct AddrGetRequest {
    if_index: Option<u32>,
}

impl AddrGetRequest {
    pub fn for_ifname(ifname: &str) -> Result<Self, Error> {
        let if_index = if_nametoindex(ifname).map_err(|e| Error::InterfaceLookup {
            ifname: ifname.to_owned(),
            source: e,
        })?;
        Ok(AddrGetRequest {
            if_index: Some(if_index),
        })
    }

    pub fn all() -> Self {
        AddrGetRequest { if_index: None }
    }

    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<AddrGetResponse, Error> {
        let ifaddrmsg = IfaddrmsgBuilder::default()
            .ifa_family(RtAddrFamily::Inet)
            .ifa_prefixlen(0)
            .ifa_scope(RtScope::Universe)
            .ifa_index(self.if_index.unwrap_or(0))
            .build()?;
        let recv: NlRouterReceiverHandle<Rtm, Ifaddrmsg> = h
            .rtnl
            .send(Rtm::Getaddr, NlmF::DUMP, NlPayload::Payload(ifaddrmsg))
            .map_err(|e| Error::Send(Box::new(e)))?;

        let mut interfaces_by_index = BTreeMap::new();

        for response in recv {
            let header: Nlmsghdr<Rtm, Ifaddrmsg> =
                response.map_err(|e| Error::Receive(Box::new(e)))?;
            if let NlPayload::Payload(p) = header.nl_payload() {
                if header.nl_type() != &Rtm::Newaddr {
                    return Err(Error::UnexpectedNlType {
                        expected: format!("{:?}", Rtm::Newaddr),
                        actual: format!("{:?}", header.nl_type()),
                    });
                }

                let if_index: u32 = *p.ifa_index();

                let mut interface = interfaces_by_index
                    .remove(&if_index)
                    .unwrap_or_else(AddrGetInterface::default);

                let family = *p.ifa_family();
                let if_name = if_indextoname(if_index)
                    .ok()
                    .and_then(|n| n.into_string().ok());
                let prefix_len: u8 = *p.ifa_prefixlen();

                println!(
                    "family: {:?} idx={if_index} if_name={:?} scope={:?} prefix={prefix_len}",
                    family,
                    if_name,
                    p.ifa_scope()
                );

                let mut local = None;
                let mut address = None;
                let mut broadcast = None;
                let mut label = None;
                let flags = AddressFlags::from(*p.ifa_flags());
                for rtattr in p.rtattrs().iter() {
                    match *rtattr.rta_type() {
                        Ifa::Local => local = Some(rtattr_to_ipv4(rtattr)?),
                        Ifa::Address => address = Some(rtattr_to_ipv4(rtattr)?),
                        Ifa::Broadcast => broadcast = Some(rtattr_to_ipv4(rtattr)?),
                        Ifa::Label => label = Some(rtattr_to_string(rtattr)?),
                        _other => {
                            //    println!("{:?}:{:?}", _other, rtattr.payload().as_ref());
                        }
                    }
                }

                interface.addresses.push(AddrGetInterfaceAddressV4 {
                    prefix_len,
                    flags,
                    local,
                    address,
                    broadcast,
                    label,
                });

                interfaces_by_index.insert(if_index, interface);
                //println!("---\n");
            }
        }

        Ok(AddrGetResponse {
            interfaces: interfaces_by_index,
        })
    }
}
