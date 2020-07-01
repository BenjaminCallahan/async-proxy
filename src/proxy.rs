use crate::general::IOStream;

/// General trait which implementing type
/// represents an asyncronous proxy client (stream)
#[async_trait::async_trait]
pub trait ProxyStream {
    /// Represents a stream that the proxy
    /// client operates on (sends protocol data over it)
    type Stream: IOStream;
    /// Used for internal proxy error indication
    type ErrorKind;
    /// Parameters that are passed to the
    /// connect function.
    /// 
    /// Each proxification protocol requires
    /// own parameters in a client implementation,
    /// so the implementing type must annotate it.
    /// 
    /// For instance, a Socks4 protocol implementation
    /// may require (if it is flexible it will actually
    ///  require) destanation IPv4 address and port,
    /// while an HTTP(s) protocol implementation may
    /// require you a destanation URI
    type ConnParams;

    /// Takes ownership of an existant stream and
    /// establishes on it connection.
    /// Returns a `ProxyStream` if the connection
    /// was successful, an error if not.
    async fn connect(stream: Self::Stream, params: Self::ConnParams)
        -> Result<Self, Self::ErrorKind>
    where
        Self: Sized;
}