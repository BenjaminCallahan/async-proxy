use crate::clients::socks5;
use crate::general::ConnectionTimeouts;
use crate::proxy::ProxyConstructor;
use byteorder::{BigEndian, ByteOrder};
use core::task::{Context, Poll};
use std::io;
use std::pin::Pin;
use std::str::FromStr;
use std::{fmt, ops::Not};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Represents the proxy constructor
/// that builds a stream when the function
/// `connect` is invoked
pub struct TcpNoAuth<'a> {
    /// Represents an address of
    /// a service to what user
    /// wants to connect through a proxy
    destination: socks5::Destination,
    /// The port of the destination service
    port: u16,
    /// Timeouts for the connection
    timeouts: ConnectionTimeouts,
    /// Type of Authentication for the connection
    /// by default Authentication is not required
    auth: AuthenticationKind<'a>,
}

// All types of authentication for the connection
// I supported only one
pub enum AuthenticationKind<'a> {
    // No Authentication required
    NoAuthentication,
    // GSSAPI
    GenericSecurityServicesAPI,
    // Authentication by username and password
    UsernamePassword {
        username: &'a str,
        password: &'a str,
    },

    PrivateMethods,

    NoAcceptable,
}

#[derive(Debug)]
/// An error that can occur when connecting
/// to a service through a socks5 proxy client
pub enum ErrorKind {
    /// Indicates that an operation
    /// took too much time so that
    /// timeouts has been reached
    OperationTimeoutReached,
    /// Indicates an I/O error
    IOError(std::io::Error),
    /// Indicates that a socks5-proxy
    /// server has replied with a bad buffer
    BadBuffer,
    /// May occur only if a destination
    /// is a domain name and indicates
    /// that the domain name is too long.
    /// The maximal length is 255
    DomainNameTooLong,
    /// Indicates that it is unable
    /// to establish a connection
    /// due to that fact that a socks5-proxy
    /// server is currently unavailable
    SocksServerFailure,
    /// Indicates that the connection request
    /// was denied due to the server rules
    RequestDenied,
    /// Indicates that it is unable to
    /// establish a connection due to that fact
    /// that the network is unreachable
    NetworkUnreachable,
    /// Indicates that a host is unreachable
    /// so that it is unable to establish a connection
    HostUnreachable,
    /// Indicates that a connection is refused
    /// so that it is unable to establish a connection
    ConnectionRefused,
    /// Indicates that it is unable to establish a connection
    /// due to that fact that a TTL is expired
    TTLExpired,
    /// Indicates that the command sent on the server
    /// is not currently supported, or the protocol
    /// is broken on a server side
    NotSupported,
    /// Indicates that the type of a destination
    /// address is not supported
    DestinationNotSupported,
    /// Indicates the the type of not supported method currently
    Method(NotSupportedMethod),
}

#[derive(Debug)]
/// This is list of methods
/// which is currently I not supported
/// Only one mothod I supported by UsernamePassword
pub enum NotSupportedMethod {
    /// Indicated that the authentication not required
    NoAuthRequired,
    /// Indicated that method for Auth it is GSSAPI
    /// Not supported
    GssAPI,
    /// Indicated that the values reserved for IANA
    IANA,
    /// Indicated that the values reserved for private methods
    PrivateMethods,
}

/// Represents an error that
/// can occur during `from_str`
/// parsing
#[derive(Debug)]
pub enum StrParsingError {
    /// Indicates that the string is not
    /// formatted appropriately for parsing
    /// process
    SyntaxError,
    /// Indicates that a destination
    /// address cannot be parsed
    InvalidDestination,
    /// Indicates that a port (u16)
    /// is invalid and it is unable to parse it
    InvalidPort,
    /// Indicates that timeouts
    /// cannot be parsed
    InvalidTimeouts,
}

/// Represents the socks5-tcp
/// proxy client stream implementation
pub struct TcpNoAuthStream {
    /// The tcp stream on which
    /// the client operates on
    wrapped_stream: TcpStream,
}

impl<'a> TcpNoAuth<'a> {
    pub fn new(
        destination: socks5::Destination,
        port: u16,
        timeouts: ConnectionTimeouts,
    ) -> TcpNoAuth<'a> {
        TcpNoAuth {
            destination,
            port,
            timeouts,
            auth: AuthenticationKind::NoAuthentication,
        }
    }

    pub fn with_authentication(&mut self, username: &'a str, password: &'a str) {
        self.auth = AuthenticationKind::UsernamePassword { username, password };
    }
}

/// Impl for parsing a `Socks4General`
/// from a string
impl<'a> FromStr for TcpNoAuth<'a> {
    type Err = StrParsingError;

    /// Parses a `Socks4General` from a
    /// string in format:
    ///   (ipv4 or ipv6 or domain.com) port timeouts
    fn from_str(s: &str) -> Result<TcpNoAuth<'a>, Self::Err> {
        // Splitting the string on spaces
        let mut s = s.split(" ");

        // Parsing an address and timeouts
        let (destination, port, timeouts) = (
            s.next()
                .ok_or(StrParsingError::SyntaxError)?
                .parse::<socks5::Destination>()
                .map_err(|_| StrParsingError::InvalidDestination)?,
            s.next()
                .ok_or(StrParsingError::SyntaxError)?
                .parse::<u16>()
                .map_err(|_| StrParsingError::InvalidPort)?,
            s.next()
                .ok_or(StrParsingError::SyntaxError)?
                .parse::<ConnectionTimeouts>()
                .map_err(|_| StrParsingError::InvalidTimeouts)?,
        );

        Ok(TcpNoAuth::new(destination, port, timeouts))
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ErrorKind::DomainNameTooLong => f.write_str("domain name is too long"),
            ErrorKind::IOError(e) => f.write_str(&format!("I/O error: {}", e)),
            ErrorKind::BadBuffer => f.write_str("bad buffer has been received"),
            ErrorKind::RequestDenied => f.write_str("request denied"),
            ErrorKind::SocksServerFailure => f.write_str("SOCKS5 server is unavailable"),
            ErrorKind::NetworkUnreachable => f.write_str("network is unreachable"),
            ErrorKind::HostUnreachable => f.write_str("destination host is unreachable"),
            ErrorKind::ConnectionRefused => f.write_str("connection refused"),
            ErrorKind::TTLExpired => f.write_str("TTL is expired"),
            ErrorKind::NotSupported => {
                f.write_str("operation is not supported by the SOCKS server")
            }
            ErrorKind::DestinationNotSupported => {
                f.write_str("the type of passed destination is not supported")
            }
            ErrorKind::OperationTimeoutReached => f.write_str("operation timeout reached"),
            ErrorKind::Method(method_kind) => match method_kind {
                NotSupportedMethod::NoAuthRequired => {
                    f.write_str("the authentication not required")
                }
                NotSupportedMethod::GssAPI => {
                    f.write_str("method of auth are GssAPI. Currently not supprted")
                }
                NotSupportedMethod::IANA => {
                    f.write_str("field of method assignd for IANA. Currently not supported")
                }
                NotSupportedMethod::PrivateMethods => f.write_str(
                    "field of method reserved for private method. Currently not supported",
                ),
            },
        }
    }
}
#[async_trait::async_trait]
impl<'a> ProxyConstructor for TcpNoAuth<'a> {
    type Stream = TcpStream;
    type ProxyStream = TcpNoAuthStream;
    type ErrorKind = ErrorKind;

    async fn connect(
        &mut self,
        mut stream: Self::Stream,
    ) -> Result<Self::ProxyStream, Self::ErrorKind> {
        // The length of the initial Socks5 request's buffer
        const BUF_LEN: usize = 3;

        // Creating the payload buffer
        let mut buf = Vec::<u8>::with_capacity(BUF_LEN);

        // The number of the Socks protocol version
        // (0x05 or just 5 in this case)
        buf.push(5);

        // The number of supported authentication methods
        // (1 in this case)
        buf.push(1);

        match self.auth {
            // The only one value of the supported
            // authentication method.
            // (0x00 or just 0 — No authentication)
            // by default uses this kind
            AuthenticationKind::NoAuthentication => buf.push(0),

            // This means user chose the kind of authentication
            // as a Username/Password
            AuthenticationKind::UsernamePassword {
                username: _,
                password: _,
            } => {
                // Add to the message for server
                // method of authentication
                // X'02' USERNAME/PASSWORD
                buf.push(2);
            }

            _ => println!("Not supported method authentication"),
        };

        // Writing the initial payload to the server
        let read_bytes = self.send_payload(&mut buf, &mut stream).await.unwrap();

        // The server must send reply
        // with the length of 2 bytes.
        // Anything else is a sense of an error
        if read_bytes != 2 {
            return Err(ErrorKind::BadBuffer);
        }

        // The former read byte must be 0x05,
        // while the latter must not be 0xFF
        if buf[0] != 0x05 || buf[1] == 0xFF {
            return Err(ErrorKind::BadBuffer);
        }
        match buf[1] {
            0x0 => return Err(ErrorKind::Method(NotSupportedMethod::NoAuthRequired)),
            0x01 => return Err(ErrorKind::Method(NotSupportedMethod::GssAPI)),

            // This means
            // method of authentication UserName/Password
            0x02 => {
                // The VER field contains the current version of the subnegotiation
                // which is X'01'
                buf[0] = 1;

                if let AuthenticationKind::UsernamePassword { username, password } = self.auth {
                    let buf_size: usize = 1 + 1 + username.len() + 1 + password.len();

                    buf.resize(buf_size, 0);

                    // The length of UNAME
                    let username_length = username.len();

                    // Set username length to the ULEN field
                    buf[1] = username_length as u8;

                    // Set username to the UNAME field
                    // (2) start index because field of UNAME start from 2
                    // and last index it is start index + length of username
                    buf[2..2 + username_length].clone_from_slice(username.as_bytes());

                    // Length of password
                    let pass_length = password.len();

                    // Set password of length to the PLEN field
                    // 2 + username_length this is index right after UNAME field
                    buf[2 + username_length] = pass_length as u8;

                    // Set password to the PASSWD field
                    // 2 + username_length + 1 this index rigth after PLEN field
                    buf[2 + username_length + 1..].clone_from_slice(password.as_bytes());

                    let read_bytes = self.send_payload(&mut buf, &mut stream).await.unwrap();

                    // The server must send reply
                    // with the length of 2 bytes.
                    // Anything else is a sense of an error
                    if read_bytes != 2 {
                        return Err(ErrorKind::BadBuffer);
                    }

                    // Analyzing the received reply
                    // and returning a socks4 general proxy client
                    // instance if everything was successful
                    return match buf[1] {
                        // Means that request accepted
                        0x00 => Ok(TcpNoAuthStream {
                            wrapped_stream: stream,
                        }),
                        0x01 => Err(ErrorKind::SocksServerFailure),
                        0x02 => Err(ErrorKind::RequestDenied),
                        0x03 => Err(ErrorKind::NetworkUnreachable),
                        0x04 => Err(ErrorKind::HostUnreachable),
                        0x05 => Err(ErrorKind::ConnectionRefused),
                        0x06 => Err(ErrorKind::TTLExpired),
                        0x07 => Err(ErrorKind::NotSupported),
                        0x08 => Err(ErrorKind::DestinationNotSupported),
                        _ => Err(ErrorKind::BadBuffer),
                    };
                }
            }
            0x03..=0x7F => return Err(ErrorKind::Method(NotSupportedMethod::IANA)),
            0x80..=0xFE => return Err(ErrorKind::Method(NotSupportedMethod::PrivateMethods)),
            0xFF => return Err(ErrorKind::BadBuffer),
        };

        // Computing the length of a Socks5 request
        // The buffer length is computed this way:
        //  (+1) for the number of the version of the socks protocol (4 in this case)
        //  (+1) for the command number (1 or 2)
        //  (+1) for the reserved byte, must be 0x00
        //  (+1) for the destination address type,
        //       must be 0x01, 0x03 or 0x04,
        //       where 0x01 stands for an IPv4 address,
        //       0x03 stands for a domain name and
        //       0x04 stands for an IPv6 address
        //  [+4]* if the type of the address is IPv4,
        //  [+n]* if the type of the address is domain
        //  [+16]* if the type of the address is IPv6
        //  (+2) for port (in the network byte order)
        let dest_buf_len = self.destination.len_as_buffer();
        let buf_len = 1 + 1 + 1 + dest_buf_len + 2;

        // Reallocating the payload buffer
        buf.resize(buf_len, 0);

        // Setting the version of the socks protocol
        // being used in the payload buffer
        buf[0] = 5;

        // Setting the tcp connection establishment command
        buf[1] = socks5::Command::TcpConnectionEstablishment as u8;

        // Setting a 0x00 byte as it is
        // rule of the socks5 protocol
        // buf[2] = 0;

        // Filling the buffer with the destiation
        self.destination.extend_buffer(&mut buf[3..]).unwrap();

        // Writing port as a big endian short
        BigEndian::write_u16(&mut buf[3 + dest_buf_len..3 + dest_buf_len + 2], self.port);

        // Sending our generated payload
        let read_bytes = self.send_payload(&mut buf, &mut stream).await.unwrap();

        // The server must send reply
        // with the length of 2 bytes.
        // Anything else is a sense of an error
        if read_bytes != 2 {
            return Err(ErrorKind::BadBuffer);
        }

        // Analyzing the received reply
        // and returning a socks4 general proxy client
        // instance if everything was successful
        match buf[1] {
            // Means that request accepted
            0x00 => Ok(TcpNoAuthStream {
                wrapped_stream: stream,
            }),
            0x01 => Err(ErrorKind::SocksServerFailure),
            0x02 => Err(ErrorKind::RequestDenied),
            0x03 => Err(ErrorKind::NetworkUnreachable),
            0x04 => Err(ErrorKind::HostUnreachable),
            0x05 => Err(ErrorKind::ConnectionRefused),
            0x06 => Err(ErrorKind::TTLExpired),
            0x07 => Err(ErrorKind::NotSupported),
            0x08 => Err(ErrorKind::DestinationNotSupported),
            _ => Err(ErrorKind::BadBuffer),
        }
    }

    /// Writing the initial payload to the server
    async fn send_payload(
        &self,
        buf: &mut Vec<u8>,
        stream: &mut Self::Stream,
    ) -> Result<usize, Self::ErrorKind> {
        let future = stream.write_all(&buf);
        let future = timeout(self.timeouts.write_timeout, future);
        let _ = future
            .await
            .map_err(|_| ErrorKind::OperationTimeoutReached)?
            .map_err(|e| ErrorKind::IOError(e))?;

        // Reading a reply from the server
        let future = stream.read(buf);
        let future = timeout(self.timeouts.read_timeout, future);
        let read_bytes = future
            .await
            .map_err(|_| ErrorKind::OperationTimeoutReached)?
            .map_err(|e| ErrorKind::IOError(e))?;

        Ok(read_bytes)
    }
}

impl AsyncRead for TcpNoAuthStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let pinned = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(pinned).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpNoAuthStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let stream = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let stream = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let stream = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(stream).poll_shutdown(cx)
    }
}

impl Into<TcpStream> for TcpNoAuthStream {
    fn into(self) -> TcpStream {
        self.wrapped_stream
    }
}
