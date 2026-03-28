use neli::err::RouterError;
use neli::types::Buffer;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Netlink socker error: {0}")]
    NetlinkSocket(#[from] neli::err::SocketError),
    #[error("IfaddrMsgBuilder error: {0}")]
    IfaddrMsgBuilder(#[from] neli::rtnl::IfaddrmsgBuilderError),
    #[error("NlRouter error: {0}")]
    NlRouterError(#[from] RouterError<u16, Buffer>),
    #[error("NlRouter misc error")]
    NlRouterMiscError,
    #[error("Unexpected NlType")]
    UnexpectedNlType,
    #[error("Deserialise error")]
    DeError,
    #[error("Send error")]
    SendError,
    #[error("Libc Error")]
    LibcError,
}
