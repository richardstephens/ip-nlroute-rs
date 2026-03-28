use std::collections::BTreeMap;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct AddrGetResponse {
    pub interfaces: BTreeMap<u32, AddrGetInterface>,
}

#[derive(Debug, Default)]
pub struct AddrGetInterface {
    pub addresses: Vec<AddrGetInterfaceAddressV4>,
}

#[derive(Debug, Default)]
pub struct AddrGetInterfaceAddressV4 {
    pub prefix_len: u8,
    pub local: Option<Ipv4Addr>,
    pub address: Option<Ipv4Addr>,
    pub broadcast: Option<Ipv4Addr>,
    pub label: Option<String>,
}
