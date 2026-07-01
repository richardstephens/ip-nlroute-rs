use crate::NetlinkRouteHandle;
use crate::error::Error;
use crate::route::{Route, RouteGetResponse, RouteProtocol};
#[cfg(all(target_os = "linux", feature = "netlink"))]
use crate::util::mappers::ip::rtattr_to_ipv4;

#[cfg(all(target_os = "linux", feature = "netlink"))]
use neli::{
    attr::Attribute,
    consts::{
        nl::NlmF,
        rtnl::{RtAddrFamily, RtScope, RtTable, Rta, Rtm, Rtn, Rtprot},
    },
    nl::{NlPayload, Nlmsghdr},
    router::synchronous::NlRouterReceiverHandle,
    rtnl::{Rtmsg, RtmsgBuilder},
};

pub struct RouteGetRequest {
    /// When set, only return routes that would match this destination address
    /// (mirrors `ip route show to match ADDR`).
    to_match: Option<std::net::Ipv4Addr>,
}

impl Default for RouteGetRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteGetRequest {
    pub fn new() -> Self {
        RouteGetRequest { to_match: None }
    }

    /// Only return routes that would match `addr` (as `ip route show to match`).
    pub fn to_match(mut self, addr: std::net::Ipv4Addr) -> Self {
        self.to_match = Some(addr);
        self
    }

    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn send(&self, _h: &mut NetlinkRouteHandle) -> Result<RouteGetResponse, Error> {
        Err(Error::NotImplemented)
    }
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn send(&self, h: &mut NetlinkRouteHandle) -> Result<RouteGetResponse, Error> {
        use nix::net::if_::if_indextoname;
        let rtmsg = RtmsgBuilder::default()
            .rtm_family(RtAddrFamily::Inet)
            .rtm_dst_len(0)
            .rtm_src_len(0)
            .rtm_tos(0)
            .rtm_table(RtTable::Unspec)
            .rtm_protocol(Rtprot::Unspec)
            .rtm_scope(RtScope::Universe)
            .rtm_type(Rtn::Unspec)
            .build()?;

        let recv: NlRouterReceiverHandle<Rtm, Rtmsg> = h
            .rtnl
            .send(Rtm::Getroute, NlmF::DUMP, NlPayload::Payload(rtmsg))
            .map_err(|e| Error::Send(Box::new(e)))?;

        let mut routes = Vec::new();

        for response in recv {
            let header: Nlmsghdr<Rtm, Rtmsg> = response.map_err(|e| Error::Receive(Box::new(e)))?;
            if let NlPayload::Payload(p) = header.nl_payload() {
                if header.nl_type() != &Rtm::Newroute {
                    return Err(Error::UnexpectedNlType {
                        expected: format!("{:?}", Rtm::Newroute),
                        actual: format!("{:?}", header.nl_type()),
                    });
                }

                if p.rtm_table() != &RtTable::Main {
                    continue;
                }

                let dst_prefix_len = *p.rtm_dst_len();
                let protocol: RouteProtocol = (*p.rtm_protocol()).into();
                let scope = (*p.rtm_scope()).into();
                let route_type = (*p.rtm_type()).into();
                let flags = (*p.rtm_flags()).into();

                let mut dst = None;
                let mut gateway = None;
                let mut prefsrc = None;
                let mut oif: Option<u32> = None;
                let mut metric = None;

                for rtattr in p.rtattrs().iter() {
                    match *rtattr.rta_type() {
                        Rta::Dst => dst = Some(rtattr_to_ipv4(rtattr)?),
                        Rta::Gateway => gateway = Some(rtattr_to_ipv4(rtattr)?),
                        Rta::Prefsrc => prefsrc = Some(rtattr_to_ipv4(rtattr)?),
                        Rta::Oif => {
                            oif = Some(rtattr.get_payload_as::<u32>().map_err(|e| {
                                Error::Deserialise {
                                    what: "output interface index",
                                    source: Box::new(e),
                                }
                            })?)
                        }
                        Rta::Priority => {
                            metric = Some(rtattr.get_payload_as::<u32>().map_err(|e| {
                                Error::Deserialise {
                                    what: "route metric",
                                    source: Box::new(e),
                                }
                            })?)
                        }
                        _ => {}
                    }
                }

                // Client-side "to match" filter, as iproute2 does: keep only
                // routes whose prefix would cover the queried address.
                if let Some(target) = self.to_match
                    && !route_covers(dst, dst_prefix_len, target)
                {
                    continue;
                }

                let oif_name =
                    oif.and_then(|idx| if_indextoname(idx).ok().and_then(|n| n.into_string().ok()));

                routes.push(Route {
                    dst,
                    dst_prefix_len,
                    gateway,
                    prefsrc,
                    oif_name,
                    protocol,
                    scope,
                    route_type,
                    metric,
                    flags,
                });
            }
        }

        Ok(RouteGetResponse { routes })
    }
}

/// Whether the route `dst`/`prefix_len` (a `None` dst meaning the default
/// route) covers `target`.
#[cfg(all(target_os = "linux", feature = "netlink"))]
fn route_covers(
    dst: Option<std::net::Ipv4Addr>,
    prefix_len: u8,
    target: std::net::Ipv4Addr,
) -> bool {
    if prefix_len == 0 {
        return true;
    }
    let mask: u32 = if prefix_len >= 32 {
        u32::MAX
    } else {
        !((1u32 << (32 - prefix_len)) - 1)
    };
    let network = u32::from(dst.unwrap_or(std::net::Ipv4Addr::UNSPECIFIED));
    (u32::from(target) & mask) == (network & mask)
}
