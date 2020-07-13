/// A general trait that represents
/// something that constructs a proxy stream,
/// something, where we can write to and read from
/// just as from a usual stream but through a proxy
#[async_trait::async_trait]
pub trait ProxyConstructor {
    /// Represents a stream that the proxy
    /// client operates on (sends protocol data over it)
    type Stream: Send;
    /// Represents the actual proxy stream,
    /// returned by the connect function
    type ProxyStream: Send;
    /// Used for internal proxy error indication
    type ErrorKind;

    /// Takes ownership of an existant stream,
    /// establishes a proxixied connection on the stream
    /// and returns the proxy stream if the connection was
    /// successful, unless an error
    async fn connect(&mut self, stream: Self::Stream)
        -> Result<Self::ProxyStream, Self::ErrorKind>
    where
        Self: Sized;
}