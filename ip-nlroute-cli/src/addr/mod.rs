use ip_nlroute::NetlinkRouteHandle;
use ip_nlroute::addr::get::AddrGetRequest;

pub fn get_addresses() -> anyhow::Result<()> {
    let mut nl = NetlinkRouteHandle::open()?;

    AddrGetRequest::send(&mut nl)?;

    Ok(())
}
