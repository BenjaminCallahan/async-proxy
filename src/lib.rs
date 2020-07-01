//! # async-proxy
//! 
//! The crate `async-proxy` provides a fast and flexible,
//! as well as asyncronous implementation of proxy clients
//! and proxy-related utilities.

/// Module responsible for functionality
/// related to proxy clients interfaces
/// (eg. common definitions and traits)
pub mod proxy;

/// Module responsible for client implementations
/// of known and most-used proxifications
/// protocols, such as Socks4/5, HTTP(s)
/// proxies
pub mod clients;