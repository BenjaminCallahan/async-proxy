//! # async-proxy
//! 
//! The crate `async-proxy` provides a fast and flexible,
//! as well as asyncronous implementation of proxy clients
//! and proxy-related utilities.

use tokio::io::{AsyncRead, AsyncWrite};

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

/// General trait which implementing type
/// represents something where we can both
/// write to or read from
pub trait IOStream: AsyncRead + AsyncWrite {}