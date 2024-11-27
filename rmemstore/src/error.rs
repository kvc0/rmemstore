#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("socket error: {0}")]
    SocketError(#[from] protosocket_rpc::Error),
    #[error("connection is broken: {0}")]
    ConnectionBroken(&'static str),
    #[error("malformed response: {0}")]
    MalformedResponse(&'static str),
}
