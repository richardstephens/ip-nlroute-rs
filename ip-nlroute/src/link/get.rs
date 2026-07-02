use crate::NetlinkRouteHandle;
use crate::error::Error;
use crate::link::get_response::{Link, LinkGetResponse};
use crate::util::mappers::interface::resolve_ifname;
#[cfg(all(target_os = "linux", feature = "netlink"))]
use neli::{
    attr::Attribute,
    consts::{
        nl::NlmF,
        rtnl::{Ifla, IflaInfo, RtAddrFamily, Rtm},
    },
    nl::{NlPayload, Nlmsghdr},
    router::synchronous::NlRouterReceiverHandle,
    rtnl::{Ifinfomsg, IfinfomsgBuilder},
};
#[cfg(all(target_os = "linux", feature = "netlink"))]
use std::collections::BTreeMap;

pub struct LinkGetRequest {
    if_index: Option<u32>,
}

impl LinkGetRequest {
    /// Request all links.
    pub fn all() -> Self {
        LinkGetRequest { if_index: None }
    }

    /// Request a single link by name.
    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn for_ifname(_ifname: &str) -> Result<Self, Error> {
        Err(Error::NotImplemented)
    }

    /// Request a single link by name.
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn for_ifname(ifname: &str) -> Result<Self, Error> {
        Ok(LinkGetRequest {
            if_index: Some(resolve_ifname(ifname)?),
        })
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<LinkGetResponse, Error> {
        Err(Error::NotImplemented)
    }

    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<LinkGetResponse, Error> {
        use nix::net::if_::if_indextoname;

        let ifinfomsg = IfinfomsgBuilder::default()
            .ifi_family(RtAddrFamily::Unspecified)
            .build()?;

        let recv: NlRouterReceiverHandle<Rtm, Ifinfomsg> = h
            .rtnl
            .send(Rtm::Getlink, NlmF::DUMP, NlPayload::Payload(ifinfomsg))
            .map_err(|e| Error::Send(Box::new(e)))?;

        let mut links = BTreeMap::new();

        for response in recv {
            let header: Nlmsghdr<Rtm, Ifinfomsg> =
                response.map_err(|e| Error::Receive(Box::new(e)))?;
            if let NlPayload::Payload(p) = header.nl_payload() {
                if header.nl_type() != &Rtm::Newlink {
                    return Err(Error::UnexpectedNlType {
                        expected: format!("{:?}", Rtm::Newlink),
                        actual: format!("{:?}", header.nl_type()),
                    });
                }

                let if_index = *p.ifi_index() as u32;

                // When filtering to a single interface, skip the rest.
                if let Some(want) = self.if_index
                    && want != if_index
                {
                    continue;
                }

                // `ifi_type` is the ARPHRD_* hardware type; recover the raw
                // value (neli's Arphrd enum omits some, e.g. WiFi) and map it.
                let hw_type = crate::link::get_response::LinkType::from(u16::from(*p.ifi_type()));
                let flags = (*p.ifi_flags()).into();

                let mut if_name = None;
                let mut mtu = None;
                let mut address = None;
                let mut broadcast = None;
                let mut master = None;
                let mut kind = None;

                for rtattr in p.rtattrs().iter() {
                    match *rtattr.rta_type() {
                        Ifla::Ifname => if_name = Some(rtattr_to_ifname(rtattr)?),
                        Ifla::Mtu => mtu = Some(rtattr_to_u32(rtattr, "link MTU")?),
                        Ifla::Address => address = Some(rtattr_to_hwaddr(rtattr)),
                        Ifla::Broadcast => broadcast = Some(rtattr_to_hwaddr(rtattr)),
                        Ifla::Master => master = Some(rtattr_to_u32(rtattr, "master index")?),
                        Ifla::Linkinfo => kind = rtattr_to_kind(rtattr),
                        _ => {}
                    }
                }

                let master_name = master
                    .and_then(|idx| if_indextoname(idx).ok().and_then(|n| n.into_string().ok()));

                links.insert(
                    if_index,
                    Link {
                        if_index,
                        if_name,
                        hw_type,
                        flags,
                        mtu,
                        address,
                        broadcast,
                        master,
                        master_name,
                        kind,
                    },
                );
            }
        }

        Ok(LinkGetResponse { links })
    }
}

#[cfg(all(target_os = "linux", feature = "netlink"))]
fn rtattr_to_ifname(
    rtattr: &neli::rtnl::Rtattr<Ifla, neli::types::Buffer>,
) -> Result<String, Error> {
    let name = crate::util::mappers::ip::rtattr_to_string(rtattr)?;
    Ok(name.trim_end_matches('\0').to_owned())
}

#[cfg(all(target_os = "linux", feature = "netlink"))]
fn rtattr_to_u32(
    rtattr: &neli::rtnl::Rtattr<Ifla, neli::types::Buffer>,
    what: &'static str,
) -> Result<u32, Error> {
    rtattr
        .get_payload_as::<u32>()
        .map_err(|e| Error::Deserialise {
            what,
            source: Box::new(e),
        })
}

#[cfg(all(target_os = "linux", feature = "netlink"))]
fn rtattr_to_hwaddr(
    rtattr: &neli::rtnl::Rtattr<Ifla, neli::types::Buffer>,
) -> crate::link::get_response::HwAddr {
    crate::link::get_response::HwAddr(rtattr.payload().as_ref().to_vec())
}

#[cfg(all(target_os = "linux", feature = "netlink"))]
fn rtattr_to_kind(rtattr: &neli::rtnl::Rtattr<Ifla, neli::types::Buffer>) -> Option<String> {
    let handle = rtattr.get_attr_handle::<IflaInfo>().ok()?;
    let kind = handle
        .get_attr_payload_as_with_len::<String>(IflaInfo::Kind)
        .ok()?;
    Some(kind.trim_end_matches('\0').to_owned())
}
