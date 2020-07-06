use crate::general::ConnectionTimeouts;
use crate::proxy::ProxyStream;
use crate::clients::socks4::Command;
use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use std::pin::Pin;
use core::task::{Poll, Context};
use std::net::SocketAddrV4;
use std::borrow::Cow;
use std::fmt;
use std::io;

/// The actual type that represents
/// the Socks4 proxy client.
/// Contains a tcp stream that operates on
pub struct Socks4General {
    wrapped_stream: TcpStream
}

/// Represents a Socks4 protocol error
/// that can occur when connecting to
/// a destination
pub enum ErrorKind {
    ConnectionFailed,
    IOError(std::io::Error),
    BadBuffer,
    RequestDenied,
    IdentIsUnavailable,
    BadIdent,
    OperationTimeoutReached
}

/// Parameters required by this Socks4
/// proxy client protocol implementation
pub struct ConnParams {
    /// the IPv4 address of a service
    /// we are connecting through proxy
    dest_addr: SocketAddrV4,
    /// An ident (see Socks4 protocol wiki
    ///  for more information)
    ident: Cow<'static, str>,
    /// The timeout set
    timeouts: ConnectionTimeouts
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ErrorKind::ConnectionFailed => f.write_str("connection failed"),
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

impl ConnParams {
    pub fn new(dest_addr: SocketAddrV4, ident: Cow<'static, str>,
               timeouts: ConnectionTimeouts)
        -> ConnParams
    {
        ConnParams { dest_addr, ident, timeouts }
    }
}

#[async_trait::async_trait]
impl ProxyStream for Socks4General {
    type Stream = TcpStream;
    type ErrorKind = ErrorKind;
    type ConnParams = ConnParams;

    async fn connect(mut stream: Self::Stream, params: Self::ConnParams)
        -> Result<Self, Self::ErrorKind>
    {
        // Computing the Socks4 buffer length.
        // The buffer length is computed this way:
        //  (+1) for the number of the version of the socks protocol (4 in this case)
        //  (+1) for the command number (1 or 2)
        //  (+2) for port (in the network byte order)
        //  (+4) for the IPv4 address
        //  (+n) where `n` is the length of the given ident
        //  (+1) for the NULL-termination byte (0x00)
        let buf_len = 1 + 1 + 2 + 4 + params.ident.len() + 1;
        // Creating the payload buffer
        let mut buf = Vec::with_capacity(buf_len);

        // Pushing the version of the socks protocol
        // being used in the payload buffer
        buf.push(4);

        // Pusing the tcp connection establishment command
        buf.push(Command::TcpConnectionEstablishment as u8);
        
        // Converting the given service port into bytes
        let port_in_bytes = params.dest_addr.port().to_be_bytes();
        // Pushing the port represented as bytes
        buf.extend_from_slice(&port_in_bytes[..]);

        // Converting the given service IPv4 address
        // into bytes
        let ipaddr_in_bytes = params.dest_addr.ip().octets();
        // Pushing the byte representation of the
        // IPv4 address
        buf.extend_from_slice(&ipaddr_in_bytes[..]);

        // Pusing the given ident
        buf.extend_from_slice(&params.ident.as_bytes());

        // And, finally, pushing the
        // NULL-termination (0x00) byte
        buf.push(0);

        // Sending our generated payload
        // to the Socks4 server
        let future = stream.write_all(&buf);
        let future = timeout(params.timeouts.write_timeout, future);
        let _ = future.await.map_err(|_| ErrorKind::OperationTimeoutReached)?
                            .map_err(|e| ErrorKind::IOError(e))?;

        // Reading a reply from the server
        let future = stream.read(&mut buf);
        let future = timeout(params.timeouts.read_timeout, future);
        let read_bytes = future.await.map_err(|_| ErrorKind::OperationTimeoutReached)?
                                     .map_err(|e| ErrorKind::IOError(e))?;

        // We should receive exatly 8 bytes from the server,
        // unless there is something wrong with the
        // received reply
        if read_bytes != 8 {
            return Err(ErrorKind::BadBuffer)
        }

        // Analyzing the received reply
        // and returning a socks4 general proxy client
        // instance if everything was successful
        match buf[1] {
            // Means that request accepted
            0x5a => Ok(Socks4General { wrapped_stream: stream }),
            // Means that our request was denied
            0x5b => Err(ErrorKind::RequestDenied),
            // Means that ident is currently unavailable
            0x5c => Err(ErrorKind::IdentIsUnavailable),
            // Means that the user passed a wrong ident string
            0x5d => Err(ErrorKind::BadIdent),
            // Does not match anything, means that
            // we got a bad buffer
            _ => Err(ErrorKind::BadBuffer)
        }
    }
}

impl AsyncRead for Socks4General {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
        -> Poll<io::Result<usize>>
    {
        let pinned = &mut Pin::into_inner(self).wrapped_stream;
        Pin::new(pinned).poll_read(cx, buf)
    }
}

impl AsyncWrite for Socks4General {
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