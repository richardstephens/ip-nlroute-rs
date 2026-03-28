use neli::err::RouterError;
use neli::types::Buffer;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("netlink socket error")]
    NetlinkSocket(#[from] neli::err::SocketError),
    #[error("failed to build address message")]
    IfaddrMsgBuilder(#[from] neli::rtnl::IfaddrmsgBuilderError),
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
    #[error("failed to resolve interface '{ifname}'")]
    InterfaceLookup {
        ifname: String,
        #[source]
        source: nix::errno::Errno,
    },
}
