#[cfg(all(target_os = "linux", feature = "netlink"))]
use neli::{err::RouterError, types::Buffer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    #[error("netlink socket error")]
    NetlinkSocket(#[from] neli::err::SocketError),
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    #[error("failed to build address message")]
    IfaddrMsgBuilder(#[from] neli::rtnl::IfaddrmsgBuilderError),
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    #[error("failed to build route message")]
    RtMsgBuilder(#[from] neli::rtnl::RtmsgBuilderError),
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    #[error("netlink router error")]
    NlRouter(#[from] RouterError<u16, Buffer>),
    #[error("failed to send netlink request")]
    Send(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("failed to receive netlink response")]
    Receive(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("unexpected netlink message type: expected {expected}, got {actual}")]
    UnexpectedNlType { expected: String, actual: String },
    #[error("failed to deserialise {what}")]
    Deserialise {
        what: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[cfg(all(target_os = "linux", feature = "netlink"))]
    #[error("failed to resolve interface '{ifname}'")]
    InterfaceLookup {
        ifname: String,
        #[source]
        source: nix::errno::Errno,
    },
    #[error("Expected exactly 1 {what}, found {len}")]
    ExpectedExactlyOne { what: &'static str, len: usize },
    #[error("Response contained invalid data: {reason}")]
    InvalidDataInResponse { reason: &'static str },
    #[error("Not implemented")]
    NotImplemented,
}
