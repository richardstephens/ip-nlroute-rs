use neli::consts::rtnl::{RtScope, RtmF, Rtn, Rtprot};
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct RouteGetResponse {
    pub routes: Vec<Route>,
}

#[derive(Debug)]
pub struct Route {
    pub dst: Option<Ipv4Addr>,
    pub dst_prefix_len: u8,
    pub gateway: Option<Ipv4Addr>,
    pub prefsrc: Option<Ipv4Addr>,
    pub oif_name: Option<String>,
    pub protocol: RouteProtocol,
    pub scope: RouteScope,
    pub route_type: RouteType,
    pub metric: Option<u32>,
    pub flags: RouteFlags,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RouteFlags {
    pub dead: bool,
    pub pervasive: bool,
    pub onlink: bool,
    pub offload: bool,
    pub linkdown: bool,
    pub unresolved: bool,
    pub trap: bool,
}

impl From<RtmF> for RouteFlags {
    fn from(value: RtmF) -> Self {
        let bits: u32 = value.into();

        // RTNH_F_* constants from linux/rtnetlink.h
        // TODO: these don't seem to appear in neli anywhere - should we submit them?
        const RTNH_F_DEAD: u32 = 1;
        const RTNH_F_PERVASIVE: u32 = 2;
        const RTNH_F_ONLINK: u32 = 4;
        const RTNH_F_OFFLOAD: u32 = 8;
        const RTNH_F_LINKDOWN: u32 = 16;
        const RTNH_F_UNRESOLVED: u32 = 32;
        const RTNH_F_TRAP: u32 = 64;

        RouteFlags {
            dead: bits & RTNH_F_DEAD != 0,
            pervasive: bits & RTNH_F_PERVASIVE != 0,
            onlink: bits & RTNH_F_ONLINK != 0,
            offload: bits & RTNH_F_OFFLOAD != 0,
            linkdown: bits & RTNH_F_LINKDOWN != 0,
            unresolved: bits & RTNH_F_UNRESOLVED != 0,
            trap: bits & RTNH_F_TRAP != 0,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteProtocol {
    Unspec,
    Redirect,
    Kernel,
    Boot,
    Static,
    Dhcp,
    Other(u8),
}

impl From<Rtprot> for RouteProtocol {
    fn from(p: Rtprot) -> Self {
        match p {
            Rtprot::Unspec => RouteProtocol::Unspec,
            Rtprot::Redirect => RouteProtocol::Redirect,
            Rtprot::Kernel => RouteProtocol::Kernel,
            Rtprot::Boot => RouteProtocol::Boot,
            Rtprot::Static => RouteProtocol::Static,
            Rtprot::UnrecognizedConst(16) => RouteProtocol::Dhcp,
            Rtprot::UnrecognizedConst(v) => RouteProtocol::Other(v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteScope {
    Universe,
    Site,
    Link,
    Host,
    Nowhere,
    Other(u8),
}

impl From<RtScope> for RouteScope {
    fn from(s: RtScope) -> Self {
        match s {
            RtScope::Universe => RouteScope::Universe,
            RtScope::Site => RouteScope::Site,
            RtScope::Link => RouteScope::Link,
            RtScope::Host => RouteScope::Host,
            RtScope::Nowhere => RouteScope::Nowhere,
            RtScope::UnrecognizedConst(v) => RouteScope::Other(v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteType {
    Unspec,
    Unicast,
    Local,
    Broadcast,
    Anycast,
    Multicast,
    Blackhole,
    Unreachable,
    Prohibit,
    Throw,
    Nat,
    Xresolve,
    Other(u8),
}

impl From<Rtn> for RouteType {
    fn from(value: Rtn) -> Self {
        match value {
            Rtn::Unspec => RouteType::Unspec,
            Rtn::Unicast => RouteType::Unicast,
            Rtn::Local => RouteType::Local,
            Rtn::Broadcast => RouteType::Broadcast,
            Rtn::Anycast => RouteType::Anycast,
            Rtn::Multicast => RouteType::Multicast,
            Rtn::Blackhole => RouteType::Blackhole,
            Rtn::Unreachable => RouteType::Unreachable,
            Rtn::Prohibit => RouteType::Prohibit,
            Rtn::Throw => RouteType::Throw,
            Rtn::Nat => RouteType::Nat,
            Rtn::Xresolve => RouteType::Xresolve,
            Rtn::UnrecognizedConst(v) => RouteType::Other(v),
        }
    }
}
