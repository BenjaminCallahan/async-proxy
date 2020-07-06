/// Holds implementation of the actual socks4 protocol
pub mod general;

/// Holds implementation of the socks4 protocol but
/// without ident being passed when establishing
/// connection
pub mod no_ident;

/// Represents a Socks4 protocol command
#[repr(u8)]
pub enum Command {
    TcpConnectionEstablishment = 1,
    TcpPortBinding
}