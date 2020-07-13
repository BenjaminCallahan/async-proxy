/// Module contains implementations
/// of the proxification protocol Socks4
/// and utilities related to the protocol
/// 
/// # Example
/// 
/// ```
/// use async_proxy::clients::socks4::no_ident::Socks4NoIdent;
/// use async_proxy::general::ConnectionTimeouts;
/// use async_proxy::proxy::ProxyConstructor;
/// use tokio::net::TcpStream;
/// use std::net::{SocketAddr, SocketAddrV4};
/// use std::time::Duration;
/// use std::process::exit;
///
/// #[tokio::main]
/// async fn main() {
///     // The address of the proxy server that
///     // will be used to connect through.
///     // (We used a random proxy from `https://hidemy.name/en/proxy-list/`)
///     let proxy_addr: SocketAddr = "104.248.63.15:30588".parse().unwrap();
///
///     // The address of the destination service
///     // that we will be connecting to through proxy.
///     // (We used a tcp echo server from `http://tcpbin.org/`)
///     let dest_addr: SocketAddrV4 = "52.20.16.20:30000".parse().unwrap();
///
///     // Setting up timeouts
///     let timeouts = ConnectionTimeouts::new(
///         // Connecting timeout
///         Duration::from_secs(8),
///         // Write timeout
///         Duration::from_secs(8),
///         // Read timeout
///         Duration::from_secs(8)
///     );
///
///     // Creating the socks4 constructor,
///     // using which we will establish a connection
///     // through proxy
///     let socks4_proxy = Socks4NoIdent::new(dest_addr, timeouts);
///
///     // Connecting to the stream and getting the readable and
///     // writable stream, or terminating the script if it is
///     // unable to connect
///     let stream = TcpStream::connect(proxy_addr)
///                            .await
///                            .expect("Unable to connect to the proxy server");
///
///     // Connecting to the service through proxy
///     let stream = match socks4_proxy.connect(stream).await {
///         Ok(stream) => {
///             // Successfully connected to the service
///             stream
///         },
///         Err(e) => {
///             // -- handling the error -- //
///             exit(1);
///         }
///     };
/// }
/// ```
pub mod socks4;

/// Module contains implementations
/// of the proxification protocol Socks5
/// and utilities related to the protocol
/// 
/// # Example
/// 
/// ```
/// use async_proxy::clients::socks5::{
///     Destination, no_auth::TcpNoAuth
/// };
/// use async_proxy::general::ConnectionTimeouts;
/// use async_proxy::proxy::ProxyConstructor;
/// use tokio::net::TcpStream;
/// use std::net::{
///     SocketAddr, Ipv4Addr
/// };
/// use std::time::Duration;
/// use std::process::exit;
/// 
/// #[tokio::main]
/// async fn main() {
///     // The address of the proxy server that
///     // will be used to connect through.
///     // (We used a random proxy from `https://hidemy.name/en/proxy-list/`)
///     let proxy_addr: SocketAddr = "72.11.148.222:56533".parse().unwrap();
/// 
///     // Setting up timeouts
///     let timeouts = ConnectionTimeouts::new(
///         // Connecting timeout
///         Duration::from_secs(8),
///         // Write timeout
///         Duration::from_secs(8),
///         // Read timeout
///         Duration::from_secs(8)
///     );
/// 
///     // The address of the destination service
///     // that we will be connecting to through proxy.
///     // (We used a tcp echo server from `http://tcpbin.org/`)
///     let dest_ipaddr: Ipv4Addr = Ipv4Addr::new(52, 20, 16, 20);
/// 
///     // The port of the destination service
///     const DEST_PORT: u16 = 30_000;
/// 
///     // Creating the socks5 constructor,
///     // using which we will establish a connection
///     // through proxy
///     let mut socks5_proxy = TcpNoAuth::new(Destination::Ipv4Addr(dest_ipaddr),
///                                           DEST_PORT, timeouts);
/// 
///     // Connecting to the stream and getting the readable and
///     // writable stream, or terminating the script if it is
///     // unable to connect
///     let stream = TcpStream::connect(proxy_addr)
///                            .await
///                            .expect("Unable to connect to the proxy server");
/// 
///     // Connecting to the service through proxy
///     let stream = match socks5_proxy.connect(stream).await {
///         Ok(stream) => {
///             // Successfully connected to the service
///             stream
///         },
///         Err(e) => {
///             // -- handling error -- //
///             exit(1);
///         }
///     };
/// 
///     // -- using `stream` -- //
/// }
/// ```

pub mod socks5;