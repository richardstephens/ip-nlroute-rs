use crate::error::Error;
use crate::link::flags::LinkFlags;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug)]
pub struct LinkGetResponse {
    pub links: BTreeMap<u32, Link>,
}

impl LinkGetResponse {
    /// Return the single link if the response contains exactly one, else error.
    pub fn get_only(&self) -> Result<&Link, Error> {
        if let Some((_, link)) = self.links.first_key_value()
            && self.links.len() == 1
        {
            Ok(link)
        } else {
            Err(Error::ExpectedExactlyOne {
                what: "link",
                len: self.links.len(),
            })
        }
    }

    pub fn links_iter(&self) -> impl Iterator<Item = &Link> {
        self.links.values()
    }
}

#[derive(Debug)]
pub struct Link {
    pub if_index: u32,
    pub if_name: Option<String>,
    pub flags: LinkFlags,
    pub mtu: Option<u32>,
    /// Hardware (MAC) address of the link.
    pub address: Option<HwAddr>,
    /// Link-layer broadcast address.
    pub broadcast: Option<HwAddr>,
    /// Index of the master interface this link is enslaved to, if any.
    pub master: Option<u32>,
    /// Name of the master interface, if it could be resolved.
    pub master_name: Option<String>,
    /// Link kind (e.g. "bridge", "veth"), from IFLA_INFO_KIND.
    pub kind: Option<String>,
}

/// A link-layer (hardware) address, rendered as colon-separated hex.
#[derive(Clone, PartialEq, Eq)]
pub struct HwAddr(pub Vec<u8>);

impl fmt::Display for HwAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex: Vec<String> = self.0.iter().map(|b| format!("{b:02x}")).collect();
        write!(f, "{}", hex.join(":"))
    }
}

impl fmt::Debug for HwAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Render as the address string rather than a byte array, even under {:#?}.
        write!(f, "{self}")
    }
}
