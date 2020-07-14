use crate::proxy::ProxyConstructor;
use crate::clients::socks5;
use crate::general::ConnectionTimeouts;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::time::timeout;
use core::task::{Poll, Context};
use byteorder::{ByteOrder, BigEndian};
use std::str::FromStr;
use std::pin::Pin;
use std::fmt;
use std::io;

/// Represents the proxy constructor
/// that builds a stream when the function
/// `connect` is invoked
pub struct TcpNoAuth {
    /// Represents an address of 
    /// a service to what user
    /// wants to connect through a proxy
    destination: socks5::Destination,
    /// The port of the destination service
    port: u16,
    /// Timeouts for the connection
    timeouts: ConnectionTimeouts
}

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
    DestinationNotSupported
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
    InvalidTimeouts
}

/// Represents the socks5-tcp 
/// proxy client stream implementation
pub struct TcpNoAuthStream {
    /// The tcp stream on which
    /// the client operates on
    wrapped_stream: TcpStream
}

impl TcpNoAuth {
    pub fn new(destination: socks5::Destination, port: u16, timeouts: ConnectionTimeouts)
        -> TcpNoAuth
    {
        TcpNoAuth { destination, port, timeouts }
    }
}

/// Impl for parsing a `Socks4General`
/// from a string
impl FromStr for TcpNoAuth {
    type Err = StrParsingError;

    /// Parses a `Socks4General` from a
    /// string in format:
    ///   (ipv4 or ipv6 or domain.com) port timeouts 
    fn from_str(s: &str) -> Result<TcpNoAuth, Self::Err> {
        // Splitting the string on spaces
        let mut s = s.split(" ");

        // Parsing an address and timeouts
        let (destination, port, timeouts) = (s.next()
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
                                                .map_err(|_| StrParsingError::InvalidTimeouts)?);

        Ok(TcpNoAuth::new(destination, port, timeouts))
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ErrorKind::DomainNameTooLong => f.write_str("domain name is too long"),
            ErrorKind::IOError(e) 
                => f.write_str(&format!("I/O error: {}", e)),
            ErrorKind::BadBuffer => f.write_str("bad buffer has been received"),
            ErrorKind::RequestDenied => f.write_str("request denied"),
            ErrorKind::SocksServerFailure => f.write_str("SOCKS5 server is unavailable"),
            ErrorKind::NetworkUnreachable => f.write_str("network is unreachable"),
            ErrorKind::HostUnreachable => f.write_str("destination host is unreachable"),
            ErrorKind::ConnectionRefused => f.write_str("connection refused"),
            ErrorKind::TTLExpired => f.write_str("TTL is expired"),
            ErrorKind::NotSupported => f.write_str("operation is not supported by the SOCKS server"),
            ErrorKind::DestinationNotSupported => f.write_str("the type of passed destination is not supported"),
            ErrorKind::OperationTimeoutReached => f.write_str("operation timeout reached")
        }
    }
}

#[async_trait::async_trait]
impl ProxyConstructor for TcpNoAuth {
    type Stream = TcpStream;
    type ProxyStream = TcpNoAuthStream;
    type ErrorKind = ErrorKind;

    async fn connect(&mut self, mut stream: Self::Stream)
        -> Result<Self::ProxyStream, Self::ErrorKind>
    {
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

        // The only one value of the supported
        // authentication method.
        // (0x00 or just 0 — No authentication)
        buf.push(0);

        // Writing the initial payload to the server
        let future = stream.write_all(&buf);
        let future = timeout(self.timeouts.write_timeout, future);
        let _ = future.await.map_err(|_| ErrorKind::OperationTimeoutReached)?
                            .map_err(|e| ErrorKind::IOError(e))?;

        // Reading a reply from the server
        let future = stream.read(&mut buf);
        let future = timeout(self.timeouts.read_timeout, future);
        let read_bytes = future.await.map_err(|_| ErrorKind::OperationTimeoutReached)?
                                      .map_err(|e| ErrorKind::IOError(e))?;

        // The server must send reply
        // with the length of 2 bytes.
        // Anything else is a sense of an error
        if read_bytes != 2 {
            return Err(ErrorKind::BadBuffer)
        }

        // The former read byte must be 0x05,
        // while the latter must not be 0xFF
        if buf[0] != 0x05 || buf[1] == 0xFF {
            return Err(ErrorKind::BadBuffer)
        }

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
        BigEndian::write_u16(&mut buf[3 + dest_buf_len .. 3 + dest_buf_len + 2], self.port);

        // println!("{:?}", buf);

        // Sending our generated payload
        // to the Socks5 server
        let future = stream.write_all(&buf);
        let future = timeout(self.timeouts.write_timeout, future);
        let _ = future.await.map_err(|_| ErrorKind::OperationTimeoutReached)?
                            .map_err(|e| ErrorKind::IOError(e))?;

        // Reading a reply from the server
        let future = stream.read(&mut buf);
        let future = timeout(self.timeouts.read_timeout, future);
        let read_bytes = future.await.map_err(|_| ErrorKind::OperationTimeoutReached)?
                                     .map_err(|e| ErrorKind::IOError(e))?;

        // We should receive exatly `buf_len` bytes from the server,
        // unless there is something wrong with the
        // received reply
        if read_bytes < 2 {
            return Err(ErrorKind::BadBuffer)
        }

        // Analyzing the received reply
        // and returning a socks4 general proxy client
        // instance if everything was successful
        match buf[1] {
            // Means that request accepted
            0x00 => Ok(TcpNoAuthStream { wrapped_stream: stream }),
            0x01 => Err(ErrorKind::SocksServerFailure),
            0x02 => Err(ErrorKind::RequestDenied),
            0x03 => Err(ErrorKind::NetworkUnreachable),
            0x04 => Err(ErrorKind::HostUnreachable),
            0x05 => Err(ErrorKind::ConnectionRefused),
            0x06 => Err(ErrorKind::TTLExpired),
            0x07 => Err(ErrorKind::NotSupported),
            0x08 => Err(ErrorKind::DestinationNotSupported),
            _ => Err(ErrorKind::BadBuffer)
        }
    }
}

impl AsyncRead for TcpNoAuthStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
        -> Poll<io::Result<usize>>
    {
        let pinned = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(pinned).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpNoAuthStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8])
        -> Poll<Result<usize, io::Error>>
    { 
        let stream = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Result<(), io::Error>>
    {
        let stream = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Result<(), io::Error>>
    {
        let stream = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(stream).poll_shutdown(cx)
    }
}