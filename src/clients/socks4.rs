/// Holds implementation of the actual socks4 protocol
pub mod general;

/// Holds implementation of the socks4 protocol but
/// without ident being passed when establishing
/// connection
pub mod no_ident;