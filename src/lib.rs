//! # async-proxy
//! 
//! The crate `async-proxy` provides a fast and flexible,
//! as well as asyncronous implementation of proxy clients
//! and proxy-related utilities.
//! 
//! # Example
//! 
//! ```
//! use async_proxy::clients::socks4::no_ident::Socks4NoIdent;
//! use async_proxy::general::ConnectionTimeouts;
//! use async_proxy::proxy::ProxyConstructor;
//! use tokio::net::TcpStream;
//! use std::net::{
//!     SocketAddr, SocketAddrV4,
//!     IpAddr, Ipv4Addr
//! };
//! use std::time::Duration;
//! use std::process::exit;
//! 
//! #[tokio::main]
//! async fn main() {
//!     // The address of the proxy server that
//!     // will be used to connect through.
//!     // (We used a random proxy from `https://hidemy.name/en/proxy-list/`)
//!     let proxy_ipaddr: Ipv4Addr = Ipv4Addr::new(104, 248, 63, 15);
//! 
//!     // The port of the proxy server
//!     let proxy_port: u16 = 30_588;
//! 
//!     // The full `SocketAddr` proxy server address representation
//!     let proxy_addr: SocketAddr = SocketAddr::new(IpAddr::V4(proxy_ipaddr), proxy_port);
//! 
//!     // Setting up timeouts
//!     let timeouts = ConnectionTimeouts::new(
//!         // Connecting timeout
//!         Duration::from_secs(8),
//!         // Write timeout
//!         Duration::from_secs(8),
//!         // Read timeout
//!         Duration::from_secs(8)
//!     );
//! 
//!     // The address of the destination service
//!     // that we will be connecting to through proxy.
//!     // (We used a tcp echo server from `http://tcpbin.org/`)
//!     let dest_ipaddr: Ipv4Addr = Ipv4Addr::new(52, 20, 16, 20);
//! 
//!     // The port of the destination service
//!     let dest_port: u16 = 30_000;
//! 
//!     // The full `SocketAddrV4` destination service address representation
//!     let dest_addr: SocketAddrV4 = SocketAddrV4::new(dest_ipaddr, dest_port);
//! 
//!     // Creating the socks4 constructor,
//!     // using which we will establish a connection
//!     // through proxy
//!     let socks4_proxy = Socks4NoIdent::new(dest_addr, timeouts);
//! 
//!     // Connecting to the stream and getting the readable and
//!     // writable stream, or terminating the script if it is
//!     // unable to connect
//!     let stream = TcpStream::connect(proxy_addr)
//!                            .await
//!                            .expect("Unable to connect to the proxy server");
//!
//!     // Connecting to the service through proxy
//!     let mut stream = match socks4_proxy.connect(stream).await {
//!         Ok(stream) => stream,
//!         Err(e) => {
//!             // -- unable to connect to the service -- //
//!             // -- handling the error -- //
//!             exit(1);
//!         }
//!     };
//! 
//!     // -- using `stream` -- //
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