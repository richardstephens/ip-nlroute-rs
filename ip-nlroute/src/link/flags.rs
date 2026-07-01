/// Flags associated with a network link (`IFF_*`).
#[derive(Default, Clone, Copy, Debug)]
pub struct LinkFlags {
    pub up: bool,
    pub broadcast: bool,
    pub loopback: bool,
    pub pointopoint: bool,
    pub running: bool,
    pub noarp: bool,
    pub promisc: bool,
    pub allmulti: bool,
    pub master: bool,
    pub slave: bool,
    pub multicast: bool,
    pub lower_up: bool,
    pub dormant: bool,
}

#[cfg(all(target_os = "linux", feature = "netlink"))]
impl From<neli::consts::rtnl::Iff> for LinkFlags {
    fn from(value: neli::consts::rtnl::Iff) -> Self {
        use neli::consts::rtnl::Iff;
        Self {
            up: value.contains(Iff::UP),
            broadcast: value.contains(Iff::BROADCAST),
            loopback: value.contains(Iff::LOOPBACK),
            pointopoint: value.contains(Iff::POINTOPOINT),
            running: value.contains(Iff::RUNNING),
            noarp: value.contains(Iff::NOARP),
            promisc: value.contains(Iff::PROMISC),
            allmulti: value.contains(Iff::ALLMULTI),
            master: value.contains(Iff::MASTER),
            slave: value.contains(Iff::SLAVE),
            multicast: value.contains(Iff::MULTICAST),
            lower_up: value.contains(Iff::LOWERUP),
            dormant: value.contains(Iff::DORMANT),
        }
    }
}
