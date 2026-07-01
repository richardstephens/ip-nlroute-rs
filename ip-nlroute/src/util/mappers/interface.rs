use crate::error::Error;
use nix::net::if_::if_nametoindex;

pub(crate) fn resolve_ifname(ifname: &str) -> Result<u32, Error> {
    if_nametoindex(ifname).map_err(|e| Error::InterfaceLookup {
        ifname: ifname.to_owned(),
        source: e,
    })
}
