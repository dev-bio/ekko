use thiserror::{Error};

#[derive(Error, Debug)]
pub enum EkkoError {
    #[error("Socket send, reason: {0}")]
    SocketSendIcmpv4(String),
    #[error("Socket send, reason: {0}")]
    SocketSendIcmpv6(String),
    #[error("Failed to create socket, reason: {0}")]
    SocketCreateIcmpv4(String),
    #[error("Failed to create socket, reason: {0}")]
    SocketCreateIcmpv6(String),
    #[error("Socket failed binding to address [{0}], reason: {1}")]
    SocketBindIpv4(String, String),
    #[error("Socket failed binding to address [{0}], reason: {1}")]
    SocketBindIpv6(String, String),
    #[error("Cannot combine address [{src:?}] (source) with [{dst:?}] (target).")]
    SocketIpMismatch { src: String, dst: String },
    #[error("Socket returned no address for responder.")]
    SocketReceiveNoIpv4,
    #[error("Socket returned no address for responder.")]
    SocketReceiveNoIpv6,
    #[error("Could not set sockets receive buffer size, reason: {0}")]
    SocketSetReceiveBufferSize(String),
    #[error("Socket failed setting non-blocking to {0}, reason: {1}")]
    SocketSetNonBlocking(bool, String),
    #[error("Could not set sockets read timeout, reason: {0}")]
    SocketSetReadTimeout(String),
    #[error("Could not set socket max hops, reason: {0}")]
    SocketSetMaxHopsIpv4(String),
    #[error("Could not set socket max hops, reason: {0}")]
    SocketSetMaxHopsIpv6(String),
    #[error("Could not set socket send buffer size, reason: {0}")]
    SocketSendBufferSize(String),
    #[error("Cannot combine address [{src:?}] (source) with [{dst:?}] (target).")]
    RequestIpMismatch { src: String, dst: String },
    #[error("Failed to read response field [{0}], reason: {1}")]
    ResponseReadField(&'static str, String),
    #[error("Failed to read request field [{0}], reason: {1}")]
    RequestReadField(&'static str, String),
    #[error("Failed to read request type [{0}], reason: {1}")]
    RequestReadIcmpv4Type(&'static str, String),
    #[error("Failed to read request type [{0}], reason: {1}")]
    RequestReadIcmpv6Type(&'static str, String),
    #[error("Failed to write request field [{0}], reason: {1}")]
    RequestWriteIcmpv4Field(&'static str, String),
    #[error("Failed to write request field [{0}], reason: {1}")]
    RequestWriteIcmpv6Field(&'static str, String),
    #[error("Failed to write request payload, reason: {0}")]
    RequestWriteIcmpv4Payload(String),
    #[error("Failed to write request payload, reason: {0}")]
    RequestWriteIcmpv6Payload(String),
}