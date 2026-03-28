use crate::error::Error;
use neli::attr::Attribute;
use neli::consts::rtnl::RtaType;
use neli::rtnl::Rtattr;
use neli::types::Buffer;
use std::net::Ipv4Addr;

pub fn rtattr_to_ipv4<T: RtaType>(rtattr: &Rtattr<T, Buffer>) -> Result<Ipv4Addr, Error> {
    let ip = Ipv4Addr::from(u32::from_be(rtattr.get_payload_as::<u32>().map_err(
        |e| Error::Deserialise {
            what: "IPv4 address",
            source: Box::new(e),
        },
    )?));
    Ok(ip)
}

pub fn rtattr_to_string<T: RtaType>(rtattr: &Rtattr<T, Buffer>) -> Result<String, Error> {
    Ok(rtattr
        .get_payload_as_with_len::<String>()
        .map_err(|e| Error::Deserialise {
            what: "string attribute",
            source: Box::new(e),
        })?)
}
