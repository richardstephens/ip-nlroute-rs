pub struct NetlinkRouteHandle {
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub(crate) rtnl: neli::router::synchronous::NlRouter,
}

impl NetlinkRouteHandle {
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    pub fn open() -> Result<Self, crate::error::Error> {
        let (rtnl, _) = neli::router::synchronous::NlRouter::connect(
            neli::consts::socket::NlFamily::Route,
            None,
            neli::utils::Groups::empty(),
        )?;
        rtnl.enable_strict_checking(true)?;

        Ok(Self { rtnl })
    }
    #[cfg(not(all(target_os = "linux", feature = "netlink")))]
    pub fn open() -> Result<Self, crate::error::Error> {
        Ok(Self {})
    }
}
