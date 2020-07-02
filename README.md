# async-proxy: fast proxy clients implementation
Async proxy is a fast and flexible, as well as asyncronous implementation of proxy clients in the Rust programming language

![crates.io](https://img.shields.io/crates/v/embedded-hal-mock.svg)
![License](https://img.shields.io/github/license/TonyGraim/async-proxy)
![Version](https://img.shields.io/badge/version-v0.1.0-blue)

## Getting started
Add the line below in your `Cargo.toml` file
```
async-proxy = "0.1.0"
```

## Protocols
Since it is the first stable version of the library (it has been created yesterday, lol), currently the only supported protocol is the `Socks4` proxification protocol.
We are working on these protocols, and, we hope, they will be released in the nearest few days
* SOCKS4 (Stable)
* SOCKS5 (Working on, WIP)
* HTTP(s) (Working on, WIP)


## Example

An example of using an async-proxy `Socks4` protocol implementation without `ident` in connection needed

```rust
use async_proxy::clients::socks4::no_ident::{
    Socks4NoIdent, ConnParams
};
use async_proxy::general::ConnectionTimeouts;
use async_proxy::proxy::ProxyStream;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::{
    SocketAddr, SocketAddrV4,
    IpAddr, Ipv4Addr
};
use std::time::Duration;
use std::process::exit;

#[tokio::main]
async fn main() {
    // The address of the proxy server that
    // will be used to connect through.
    // (We used a random proxy from `https://hidemy.name/en/proxy-list/`)
    let proxy_ipaddr: Ipv4Addr = Ipv4Addr::new(104, 248, 63, 15);

    // The port of the proxy server
    let proxy_port: u16 = 30_588;

    // The full `SocketAddr` proxy server address representation
    let proxy_addr: SocketAddr = SocketAddr::new(IpAddr::V4(proxy_ipaddr), proxy_port);

    // Setting up timeouts
    let timeouts = ConnectionTimeouts::new(
        // Connecting timeout
        Duration::from_secs(8),
        // Write timeout
        Duration::from_secs(8),
        // Read timeout
        Duration::from_secs(8)
    );

    // The address of the destination service
    // that we will be connecting to through proxy.
    // (We used a tcp echo server from `http://tcpbin.org/`)
    let dest_ipaddr: Ipv4Addr = Ipv4Addr::new(52, 20, 16, 20);

    // The port of the destination service
    let dest_port: u16 = 30_000;

    // The full `SocketAddrV4` destination service address representation
    let dest_addr: SocketAddrV4 = SocketAddrV4::new(dest_ipaddr, dest_port);

    // Creating required connection parameters
    // for Socks4 proxy client
    let connection_params = ConnParams::new(dest_addr, timeouts);

    // Printing out information that we are starting
    // a connection to the Socks4 proxy server
    println!("Starting connection to the Socks4 proxy server `{}`", proxy_addr);

    // Connecting to the stream and getting the readable and
    // writable stream, or terminating the script if it is
    // unable to connect
    let stream = TcpStream::connect(proxy_addr)
                           .await
                           .expect("Unable to connect to the proxy server");


    // Printing out information that we are starting
    // a connection to the service through the proxy client
    println!("Starting connection to the destination `{}` throught socks4 proxy `{}`",
              dest_addr, proxy_addr);

    // Connecting to the service through proxy
    let mut stream = match Socks4NoIdent::connect(stream, connection_params).await {
        Ok(stream) => {
            println!("Successfully connected to the service through the proxy");
            stream
        },
        Err(e) => {
            println!("Cannot connect to the service: {}", e);
            exit(1);
        }
    };

    // Getting a message that will be sent to the service
    println!("Please inter a message to be sent. Message: ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
                    .expect("Unable to read a line from stdin");

    // Sending the message to the service
    // with the timeout of 8 seconds
    let future = stream.write_all(input.as_bytes());
    let future = timeout(Duration::from_secs(8), future);
    future.await.expect("Timeout of 8 seconds reached")
                .expect("Unable to send the message");

    // Receiving a message from the service
    // with the timeout of 8 seconds
    let future = stream.read_to_string(&mut input);
    let future = timeout(Duration::from_secs(8), future);
    future.await.expect("Timeout of 8 seconds reached")
                .expect("Unable to receive a string from the service");

    // Successfully received a message.
    // Printing it out
    println!("Received message from the service: {}", input);
}
```

More examples can be found [here](https://github.com/TonyGraim/async-proxy/tree/develop/examples)
