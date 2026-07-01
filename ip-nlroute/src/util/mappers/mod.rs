#[cfg(all(target_os = "linux", feature = "netlink"))]
pub(crate) mod interface;
#[cfg(all(target_os = "linux", feature = "netlink"))]
pub(crate) mod ip;
