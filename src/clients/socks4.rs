use std::fmt;

/// Holds implementation of the actual socks4 protocol
pub mod general;

/// Holds implementation of the socks4 protocol but
/// without ident being passed when establishing
/// connection
pub mod no_ident;

pub use general::Socks4General;
pub use no_ident::Socks4NoIdent;

/// Represents a Socks4 protocol command
#[repr(u8)]
pub enum Command {
    TcpConnectionEstablishment = 1,
    TcpPortBinding
}

/// Represents a Socks4 protocol error
/// that can occur when connecting to
/// a destination
pub enum ErrorKind {
    /// Indicates that an error occured
    /// during a native I/O operation,
    /// such as writing to or reading from
    /// a stream
    IOError(std::io::Error),
    /// Indicates that a bad (wrong, not readable by Socks4)
    /// buffer is received
    BadBuffer,
    /// Indicates that the request (for ex., for connection)
    /// is denied
    RequestDenied,
    /// Indicates that the `Ident` service
    /// is not available on the server side
    IdentIsUnavailable,
    /// Indicates that a bad ident is passed
    /// in a payload so that the server refused
    /// a connection request
    BadIdent,
    /// Indicates that a timeouts has been reached
    /// when connecting to a service
    OperationTimeoutReached
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ErrorKind::IOError(e) 
                => f.write_str(&format!("I/O error: {}", e)),
            ErrorKind::BadBuffer => f.write_str("bad buffer has been received"),
            ErrorKind::RequestDenied => f.write_str("request denied"),
            ErrorKind::IdentIsUnavailable => f.write_str("ident is unavailable"),
            ErrorKind::BadIdent => f.write_str("bad ident"),
            ErrorKind::OperationTimeoutReached => f.write_str("operation timeout reached")
        }
    }
}