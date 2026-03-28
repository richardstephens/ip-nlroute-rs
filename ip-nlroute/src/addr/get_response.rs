use super::AddressFlags;
use crate::error::Error;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct AddrGetResponse {
    pub interfaces: BTreeMap<u32, AddrGetInterface>,
}

impl AddrGetResponse {
    pub fn get_only(&self) -> Result<&AddrGetInterface, Error> {
        if let Some((_, iface)) = self.interfaces.first_key_value()
            && self.interfaces.len() == 1
        {
            Ok(iface)
        } else {
            Err(Error::ExpectedExactlyOne {
                what: "interface",
                len: self.interfaces.len(),
            })
        }
    }

    pub fn interfaces_iter(&self) -> impl Iterator<Item = &AddrGetInterface> {
        self.interfaces.values()
    }
}

#[derive(Debug, Default)]
pub struct AddrGetInterface {
    pub(crate) addresses: Vec<AddrGetInterfaceAddressV4>,
}

impl AddrGetInterface {
    pub fn addresses_v4(&self) -> &[AddrGetInterfaceAddressV4] {
        self.addresses.as_slice()
    }
}

#[derive(Debug, Default)]
pub struct AddrGetInterfaceAddressV4 {
    // these are named here for what netlink calls them.
    pub(crate) local: Option<Ipv4Addr>,
    pub(crate) address: Option<Ipv4Addr>,

    pub prefix_len: u8,
    pub flags: AddressFlags,
    pub broadcast: Option<Ipv4Addr>,
    pub label: Option<String>,
}

impl AddrGetInterfaceAddressV4 {
    pub fn ip(&self) -> Option<Ipv4Addr> {
        if let Some(ip) = self.local {
            Some(ip)
        } else {
            self.address
        }
    }

    pub fn peer_ip(&self) -> Option<Ipv4Addr> {
        if let (Some(local), Some(address)) = (self.local, self.address)
            && local != address
        {
            Some(address)
        } else {
            None
        }
    }
}
