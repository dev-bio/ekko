use thiserror::{Error};

#[derive(Error, Debug)]
pub enum EkkoError {
    #[error("Socket send, reason: {0}")]
    SocketSend(String),
    #[error("Failed to create socket, reason: {0}")]
    SocketCreateIcmpv4(String),
    #[error("Failed to create socket, reason: {0}")]
    SocketCreateIcmpv6(String),
    #[error("Socket failed binding to address '{0}', reason: {1}")]
    SocketBindIpv4(String, String),
    #[error("Socket failed binding to address '{0}', reason: {1}")]
    SocketBindIpv6(String, String),
    #[error("Socket returned no address for responder.")]
    SocketReceiveNoIpv4,
    #[error("Socket returned no address for responder.")]
    SocketReceiveNoIpv6,
    #[error("Cannot combine address '{src:?}' (source) with '{tgt:?}' (target).")]
    SocketIpMismatch { src: String, tgt: String },
    #[error("Could not set sockets receive buffer size, reason: {0}")]
    SocketSetReceiveBufferSize(String),
    #[error("Could not set sockets read timeout, reason: {0}")]
    SocketSetReadTimeout(String),
    #[error("Could not set sockets max hops, reason: {0}")]
    SocketSetMaxHops(String),
    #[error("Failed to read response field '{0}', reason: {1}")]
    ResponseReadField(&'static str, String),
    #[error("Failed to read request field '{0}', reason: {1}")]
    RequestReadField(&'static str, String),
    #[error("Failed to write Icmpv4 request field '{0}', reason: {1}")]
    RequestWriteIcmpv4Field(&'static str, String),
    #[error("Failed to write Icmpv6 request field '{0}', reason: {1}")]
    RequestWriteIcmpv6Field(&'static str, String),
    #[error("Failed to write request payload, reason: {0}")]
    RequestWriteIcmpv4Payload(String),
    #[error("Failed to write request payload, reason: {0}")]
    RequestWriteIcmpv6Payload(String),
    #[error("Failed to create resolver, reason: {0}")]
    ResolverCreate(String),
    #[error("Failed to resolve address for hostname: '{0}'")]
    ResolverIpLookup(String),
    #[error("Failed to resolve domain for address: '{0}'")]
    ResolverDomainLookup(String),
    #[error("Failed to retrieve domain from cache for address: '{0}'")]
    ResolverDomainCacheLookup(String),
}
