use tokio::io::{AsyncRead, AsyncWrite};
use std::time::Duration;

/// General trait which implementing type
/// represents something where we can both
/// write to or read from
pub trait IOStream: AsyncRead + AsyncWrite + Unpin {}

/// Auto-impl for types that satisfies
/// the trait `IOStream` requirements
/// (`AsyncRead` and `AsyncWrite`)
impl<T> IOStream for T
where
    T: AsyncRead + AsyncWrite + Unpin {}

/// Just a structure containing 
/// connecting/read/write timeouts
pub struct ConnectionTimeouts {
    pub connecting_timeout: Duration,
    pub write_timeout: Duration,
    pub read_timeout: Duration
}

impl ConnectionTimeouts {
    pub fn new(connecting_timeout: Duration,
               write_timeout: Duration,
               read_timeout: Duration)
        -> ConnectionTimeouts
    {
        ConnectionTimeouts { 
            connecting_timeout,
            write_timeout,
            read_timeout
        }
    }
}