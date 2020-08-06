# async-proxy: fast proxy clients implementation
Async proxy is a fast and flexible, as well as asyncronous implementation of proxy clients in the Rust programming language

![crates.io](https://img.shields.io/crates/v/async-proxy.svg)
![License](https://img.shields.io/github/license/TonyGraim/async-proxy)
![Version](https://img.shields.io/badge/version-v0.2.4-blue)

## Getting started
Add the line below in your `Cargo.toml` file
```
async-proxy = "0.2.4"
```

## Protocols
The library supports these protocols
* SOCKS4 (Stable)
* SOCKS5 (Without auth, stable)
* HTTP(s) (Working on, WIP)


## Example

An example of using an async-proxy `Socks4` protocol implementation without `ident` in connection needed

```rust
use async_proxy::clients::socks4::no_ident::Socks4NoIdent;
use async_proxy::general::ConnectionTimeouts;
use async_proxy::proxy::ProxyConstructor;
use tokio::net::TcpStream;
use std::net::{SocketAddr, SocketAddrV4};
use std::time::Duration;
use std::process::exit;

#[tokio::main]
async fn main() {
    // The address of the proxy server that
    // will be used to connect through.
    // (We used a random proxy from `https://hidemy.name/en/proxy-list/`)
    let proxy_addr: SocketAddr = "104.248.63.15:30588".parse().unwrap();

    // The address of the destination service
    // that we will be connecting to through proxy.
    // (We used a tcp echo server from `http://tcpbin.org/`)
    let dest_addr: SocketAddrV4 = "52.20.16.20:30000".parse().unwrap();

    // Setting up timeouts
    let timeouts = ConnectionTimeouts::new(
        // Connecting timeout
        Duration::from_secs(8),
        // Write timeout
        Duration::from_secs(8),
        // Read timeout
        Duration::from_secs(8)
    );

    // Creating the socks4 constructor,
    // using which we will establish a connection
    // through proxy
    let socks4_proxy = Socks4NoIdent::new(dest_addr, timeouts);

    // Connecting to the stream and getting the readable and
    // writable stream, or terminating the script if it is
    // unable to connect
    let stream = TcpStream::connect(proxy_addr)
                           .await
                           .expect("Unable to connect to the proxy server");

    // Connecting to the service through proxy
    let stream = match socks4_proxy.connect(stream).await {
        Ok(stream) => {
            // Successfully connected to the service
            stream
        },
        Err(e) => {
            // -- handling the error -- //
            exit(1);
        }
    };
}
```

More examples can be found [here](https://github.com/TonyGraim/async-proxy/tree/master/examples)
