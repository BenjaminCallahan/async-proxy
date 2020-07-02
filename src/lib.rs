//! # async-proxy
//! 
//! The crate `async-proxy` provides a fast and flexible,
//! as well as asyncronous implementation of proxy clients
//! and proxy-related utilities.
//! 
//! # Example
//! 
//! ```
//! use async_proxy::clients::socks4::no_ident::{
//!    Socks4NoIdent, ConnParams
//!};
//!use async_proxy::general::ConnectionTimeouts;
//!use async_proxy::proxy::ProxyStream;
//!use tokio::net::TcpStream;
//!use std::net::{
//!    SocketAddr, SocketAddrV4,
//!    IpAddr, Ipv4Addr
//!};
//!use std::time::Duration;
//!
//!#[tokio::main]
//!async fn main() {
//!    // The address of the proxy server that
//!    // will be used to connect through.
//!    // (We used a random proxy from `https://hidemy.name/en/proxy-list/`)
//!    const PROXY_IPADDR: Ipv4Addr = Ipv4Addr::new(104, 248, 63, 15);
//!
//!    // The port of the proxy server
//!    const PROXY_PORT: u16 = 30_588;
//!
//!    // The full `SocketAddr` proxy server address representation
//!    let proxy_addr: SocketAddr = SocketAddr::new(IpAddr::V4(PROXY_IPADDR), PROXY_PORT);
//!
//!    // Setting up timeouts
//!    let timeouts = ConnectionTimeouts::new(
//!        // Connecting timeout
//!        Duration::from_secs(8),
//!        // Write timeout
//!        Duration::from_secs(8),
//!        // Read timeout
//!        Duration::from_secs(8)
//!    );
//!
//!   // The address of the destination service
//!    // that we will be connecting to through proxy.
//!    // (We used a tcp echo server from `http://tcpbin.org/`)
//!    const DEST_IPADDR: Ipv4Addr = Ipv4Addr::new(52, 20, 16, 20);
//!
//!    // The port of the destination service
//!    let DEST_PORT: u16 = 30_000;
//!
//!    // The full `SocketAddrV4` destination service address representation
//!    let dest_addr: SocketAddrV4 = SocketAddrV4::new(DEST_IPADDR, DEST_PORT);
//!
//!    // Creating required connection parameters
//!    // for Socks4 proxy client
//!    let connection_params = ConnParams::new(dest_addr, timeouts);
//!
//!    // `Socks4NoIdent` performs operations on
//!    // an existant stream, so we need to connect
//!    // to the proxy server by ourselves
//!    let stream = TcpStream::connect(proxy_addr)
//!                           .await
//!                           .expect("Unable to connect to the proxy server");
//!
//!    // Connecting to the service through proxy.
//!    // If connection succeed, `Socks4NoIdent::connect` returns
//!    // both a readable and writeable stream that you will
//!    // be working on as on a normal tcp connection stream
//!    let stream = match Socks4NoIdent::connect(stream, connection_params).await {
//!        Ok(stream) => stream /* Successfully connected to the service*/,
//!        Err(error) => {
//!            /* Connection error */
//!            // -- snip -- //
//!        }
//!    };
//! 
//!    // -- using stream --
//! }
//! ```
//!


/// Module responsible for functionality
/// related to proxy clients interfaces
/// (eg. common definitions and traits)
pub mod proxy;

/// Module responsible for client implementations
/// of known and most-used proxifications
/// protocols, such as Socks4/5, HTTP(s)
/// proxies
pub mod clients;

/// Module contains types and definitions
/// that are widely and generally used
/// over the library
pub mod general;