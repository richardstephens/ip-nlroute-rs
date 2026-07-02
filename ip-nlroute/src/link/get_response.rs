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
    /// Hardware type of the link (`ifi_type`, i.e. the ARPHRD_* value).
    pub hw_type: LinkType,
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

/// Hardware type of a link, from the `ifi_type` (ARPHRD_*) field. (`linux/if_arp.h`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    Netrom,            // ARPHRD_NETROM (0)
    Ether,             // ARPHRD_ETHER (1) - Ethernet
    Eether,            // ARPHRD_EETHER (2) - experimental Ethernet
    Ax25,              // ARPHRD_AX25 (3) - AX.25 amateur radio
    Pronet,            // ARPHRD_PRONET (4)
    Chaos,             // ARPHRD_CHAOS (5)
    Ieee802,           // ARPHRD_IEEE802 (6) - Token Ring / 802.2
    Arcnet,            // ARPHRD_ARCNET (7)
    Appletlk,          // ARPHRD_APPLETLK (8) - AppleTalk
    Dlci,              // ARPHRD_DLCI (15) - Frame Relay DLCI
    Atm,               // ARPHRD_ATM (19)
    Metricom,          // ARPHRD_METRICOM (23)
    Ieee1394,          // ARPHRD_IEEE1394 (24) - FireWire
    Eui64,             // ARPHRD_EUI64 (27)
    Infiniband,        // ARPHRD_INFINIBAND (32)
    Slip,              // ARPHRD_SLIP (256)
    Cslip,             // ARPHRD_CSLIP (257)
    Slip6,             // ARPHRD_SLIP6 (258)
    Cslip6,            // ARPHRD_CSLIP6 (259)
    Rsrvd,             // ARPHRD_RSRVD (260)
    Adapt,             // ARPHRD_ADAPT (264)
    Rose,              // ARPHRD_ROSE (270)
    X25,               // ARPHRD_X25 (271) - CCITT X.25
    Hwx25,             // ARPHRD_HWX25 (272)
    Can,               // ARPHRD_CAN (280) - Controller Area Network
    Mctp,              // ARPHRD_MCTP (290)
    Ppp,               // ARPHRD_PPP (512)
    Cisco,             // ARPHRD_CISCO / ARPHRD_HDLC (513) - Cisco HDLC
    Lapb,              // ARPHRD_LAPB (516)
    Ddcmp,             // ARPHRD_DDCMP (517)
    RawHdlc,           // ARPHRD_RAWHDLC (518)
    RawIp,             // ARPHRD_RAWIP (519) - raw IP (e.g. WWAN modems)
    Tunnel,            // ARPHRD_TUNNEL (768) - IPIP tunnel
    Tunnel6,           // ARPHRD_TUNNEL6 (769) - IP6IP6 tunnel
    Frad,              // ARPHRD_FRAD (770)
    Skip,              // ARPHRD_SKIP (771)
    Loopback,          // ARPHRD_LOOPBACK (772)
    Localtlk,          // ARPHRD_LOCALTLK (773)
    Fddi,              // ARPHRD_FDDI (774)
    Bif,               // ARPHRD_BIF (775)
    Sit,               // ARPHRD_SIT (776) - IPv6-in-IPv4
    Ipddp,             // ARPHRD_IPDDP (777)
    Ipgre,             // ARPHRD_IPGRE (778) - GRE over IPv4
    Pimreg,            // ARPHRD_PIMREG (779)
    Hippi,             // ARPHRD_HIPPI (780)
    Ash,               // ARPHRD_ASH (781)
    Econet,            // ARPHRD_ECONET (782)
    Irda,              // ARPHRD_IRDA (783)
    Fcpp,              // ARPHRD_FCPP (784) - Fibre Channel point-to-point
    Fcal,              // ARPHRD_FCAL (785) - Fibre Channel arbitrated loop
    Fcpl,              // ARPHRD_FCPL (786) - Fibre Channel public loop
    Fcfabric,          // ARPHRD_FCFABRIC (787) - Fibre Channel fabric
    Ieee802Tr,         // ARPHRD_IEEE802_TR (800)
    Ieee80211,         // ARPHRD_IEEE80211 (801) - WiFi
    Ieee80211Prism,    // ARPHRD_IEEE80211_PRISM (802)
    Ieee80211Radiotap, // ARPHRD_IEEE80211_RADIOTAP (803)
    Ieee802154,        // ARPHRD_IEEE802154 (804)
    Ieee802154Monitor, // ARPHRD_IEEE802154_MONITOR (805)
    Phonet,            // ARPHRD_PHONET (820)
    PhonetPipe,        // ARPHRD_PHONET_PIPE (821)
    Caif,              // ARPHRD_CAIF (822)
    Ip6gre,            // ARPHRD_IP6GRE (823) - GRE over IPv6
    Netlink,           // ARPHRD_NETLINK (824)
    SixLowpan,         // ARPHRD_6LOWPAN (825)
    Vsockmon,          // ARPHRD_VSOCKMON (826)
    None,              // ARPHRD_NONE (0xFFFE) - no link-layer header, e.g. tun
    Void,              // ARPHRD_VOID (0xFFFF)
    /// Any ARPHRD value not named above, preserved verbatim.
    Other(u16),
}

impl From<u16> for LinkType {
    fn from(value: u16) -> Self {
        // Constants from linux/if_arp.h.
        match value {
            0 => LinkType::Netrom,
            1 => LinkType::Ether,
            2 => LinkType::Eether,
            3 => LinkType::Ax25,
            4 => LinkType::Pronet,
            5 => LinkType::Chaos,
            6 => LinkType::Ieee802,
            7 => LinkType::Arcnet,
            8 => LinkType::Appletlk,
            15 => LinkType::Dlci,
            19 => LinkType::Atm,
            23 => LinkType::Metricom,
            24 => LinkType::Ieee1394,
            27 => LinkType::Eui64,
            32 => LinkType::Infiniband,
            256 => LinkType::Slip,
            257 => LinkType::Cslip,
            258 => LinkType::Slip6,
            259 => LinkType::Cslip6,
            260 => LinkType::Rsrvd,
            264 => LinkType::Adapt,
            270 => LinkType::Rose,
            271 => LinkType::X25,
            272 => LinkType::Hwx25,
            280 => LinkType::Can,
            290 => LinkType::Mctp,
            512 => LinkType::Ppp,
            513 => LinkType::Cisco,
            516 => LinkType::Lapb,
            517 => LinkType::Ddcmp,
            518 => LinkType::RawHdlc,
            519 => LinkType::RawIp,
            768 => LinkType::Tunnel,
            769 => LinkType::Tunnel6,
            770 => LinkType::Frad,
            771 => LinkType::Skip,
            772 => LinkType::Loopback,
            773 => LinkType::Localtlk,
            774 => LinkType::Fddi,
            775 => LinkType::Bif,
            776 => LinkType::Sit,
            777 => LinkType::Ipddp,
            778 => LinkType::Ipgre,
            779 => LinkType::Pimreg,
            780 => LinkType::Hippi,
            781 => LinkType::Ash,
            782 => LinkType::Econet,
            783 => LinkType::Irda,
            784 => LinkType::Fcpp,
            785 => LinkType::Fcal,
            786 => LinkType::Fcpl,
            787 => LinkType::Fcfabric,
            800 => LinkType::Ieee802Tr,
            801 => LinkType::Ieee80211,
            802 => LinkType::Ieee80211Prism,
            803 => LinkType::Ieee80211Radiotap,
            804 => LinkType::Ieee802154,
            805 => LinkType::Ieee802154Monitor,
            820 => LinkType::Phonet,
            821 => LinkType::PhonetPipe,
            822 => LinkType::Caif,
            823 => LinkType::Ip6gre,
            824 => LinkType::Netlink,
            825 => LinkType::SixLowpan,
            826 => LinkType::Vsockmon,
            0xFFFE => LinkType::None,
            0xFFFF => LinkType::Void,
            other => LinkType::Other(other),
        }
    }
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
