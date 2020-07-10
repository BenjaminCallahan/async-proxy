use byteorder::{ByteOrder, BigEndian};

/// Module contains implementation of
/// the socks5 proxification protocol
/// with no authentification required
/// when establishing a connection
/// between a client and a socks5 server
pub mod no_auth;

/// The Socks5 protocol command representation
#[repr(C)]
pub enum Command {
    TcpConnectionEstablishment = 1,
    TcpPortBinding,
    UdpPortBinding
}

/// Represents a destination address of
/// a service, to which a user wants to
/// connect through a socks5 proxy.
/// It is a good solution, but not
/// the fastest, so it will be rewritten in the
/// future in preference to a dispatch mechanism
pub enum Destination {
    /// Represents an IPv4 address
    Ipv4Addr(std::net::Ipv4Addr),
    /// Represents a domain name
    DomainName(std::borrow::Cow<'static, str>),
    /// Represents an IPv6 address
    Ipv6Addr(std::net::Ipv6Addr)
}

impl Destination {
    /// Returns the length in bytes 
    /// of the destination, represented as a buffer
    pub fn len_as_buffer(&self) -> usize {
        match self {
            Destination::Ipv4Addr(_) => 4 + 1,
            Destination::DomainName(name) => name.len() + 2,
            Destination::Ipv6Addr(_) => 16 + 1
        }
    }

    /// Extends buffer with a buffer
    /// representation of a Destination
    /// (See the Socks5 wiki for more information).
    ///
    /// Note:
    ///     I wanted to make this function generic, such as
    ///     it would have the signature like that:
    ///     ```
    ///     fn extend_buffer(buf: impl AsMut<u8>)
    ///     ```
    ///     but it was preffered to don't to it, because the sense of this
    ///     flexibility will lead to longer compilation time, and that
    ///     is totally okay in most of cases, but the function is not even
    ///     `pub(crate)`, so the choice is obvious
    ///
    fn extend_buffer(self, buf: &mut [u8])
        -> Result<(), ()>
    {
        match self {
            Destination::Ipv4Addr(addr) => {
                // If the destination is an IPv4 address, then
                // the first byte of the buffer will
                // contain `0x01`
                buf[0] = 0x01;

                // Then we need represent the IPv4
                // address as a buffer (in the network byte order)
                // and copy it to our buffer `buf`
                BigEndian::write_u32(&mut buf[1..5], addr.into());
            },
            Destination::DomainName(name) => {
                // If the destination is a domain name, then
                // the first byte of the buffer will
                // contain `0x03`
                buf[0] = 0x03;

                // Then we need to compute the length
                // of the domain name and store it
                // as a next byte in the buffer.
                // The length cannot be larger than
                // the maxumim value of a byte (0xFF or 255),
                // so we need to make sure of it

                if name.len() > 255 {
                    return Err(())
                }

                // Storing the length
                buf[1] = name.len() as u8;

                // Then the socks5 protocol requires us to 
                // represent the domain name address as
                // a buffer and copy it to our buffer `buf`
                buf[2..].clone_from_slice(name.as_bytes());
            },
            Destination::Ipv6Addr(addr) => {
                // If the destination is an IPv6 address, then
                // the first byte of the buffer will
                // contain `0x04`
                buf[0] = 0x04;

                // Then we need represent the IPv4
                // address as a buffer (in the network byte order)
                // and copy it to our buffer `buf`
                BigEndian::write_u128(&mut buf[1..17], addr.into());
            }
        }

        Ok(())
    }
}