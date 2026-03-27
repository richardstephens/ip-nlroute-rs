use neli::consts::socket::NlFamily;
use neli::router::synchronous::NlRouter;
use neli::utils::Groups;

pub struct NetlinkRouteHandle {
    pub(crate) rtnl: NlRouter,
}

impl NetlinkRouteHandle {
    pub fn open() -> Result<Self, crate::error::Error> {
        let (rtnl, _) = NlRouter::connect(NlFamily::Route, None, Groups::empty())?;
        rtnl.enable_strict_checking(true)?;

        Ok(Self { rtnl })
    }
}
