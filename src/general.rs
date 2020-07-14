use tokio::io::{AsyncRead, AsyncWrite};
use std::str::FromStr;
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
#[derive(Clone)]
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

/// Parses connection timeouts in format
/// "connection_timeout:read_timeout:write_timeout"
/// where all timeouts are values represent milliseconds
/// duration as u64
impl FromStr for ConnectionTimeouts {
    type Err = ();

    fn from_str(s: &str) -> Result<ConnectionTimeouts, Self::Err> {
        // Splitting the string on ':' to parse
        // timeouts from them
        let mut s = s.split(":");

        // Extracting values in order:
        // connection timeout, read timeout, write timeout
        let (ct, rt, wt) = (
            s.next()
             .map(|v| v.parse::<u64>()
                       .map_err(|_| ()))
             .ok_or(())??, 
            s.next()
             .map(|v| v.parse::<u64>()
                       .map_err(|_| ()))
             .ok_or(())??,
            s.next()
             .map(|v| v.parse::<u64>()
                       .map_err(|_| ()))
             .ok_or(())??
        );

        // Converting the parsed values
        // into the approrpiate durations
        Ok(ConnectionTimeouts::new(
            Duration::from_millis(ct),
            Duration::from_millis(rt),
            Duration::from_millis(wt)
        ))
    }
}