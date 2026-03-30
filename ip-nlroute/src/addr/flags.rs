/// Flags associated with an interface address.
#[derive(Default, Clone, Copy, Debug)]
pub struct AddressFlags {
    pub secondary: bool,
    pub nodad: bool,
    pub optimistic: bool,
    pub dadfailed: bool,
    pub homeaddress: bool,
    pub deprecated: bool,
    pub tentative: bool,
    pub permanent: bool,
}

#[cfg(all(target_os = "linux", feature = "netlink"))]
impl From<neli::consts::rtnl::IfaF> for AddressFlags {
    fn from(value: neli::consts::rtnl::IfaF) -> Self {
        use neli::consts::rtnl::IfaF;
        Self {
            secondary: value.contains(IfaF::SECONDARY),
            nodad: value.contains(IfaF::NODAD),
            optimistic: value.contains(IfaF::OPTIMISTIC),
            dadfailed: value.contains(IfaF::DADFAILED),
            homeaddress: value.contains(IfaF::HOMEADDRESS),
            deprecated: value.contains(IfaF::DEPRECATED),
            tentative: value.contains(IfaF::TENTATIVE),
            permanent: value.contains(IfaF::PERMANENT),
        }
    }
}
