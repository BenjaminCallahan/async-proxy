use async_proxy::clients::socks5::{
    Destination, no_auth::TcpNoAuth
};
use async_proxy::general::ConnectionTimeouts;
use async_proxy::proxy::ProxyConstructor;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::{
    SocketAddr, Ipv4Addr
};
use std::time::Duration;
use std::process::exit;

#[tokio::main]
async fn main() {
    // The address of the proxy server that
    // will be used to connect through.
    // (We used a random proxy from `https://hidemy.name/en/proxy-list/`)
    let proxy_addr: SocketAddr = "72.11.148.222:56533".parse().unwrap();

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
    const DEST_PORT: u16 = 30_000;

    // Creating the socks5 constructor,
    // using which we will establish a connection
    // through proxy
    let socks5_proxy = TcpNoAuth::new(Destination::Ipv4Addr(dest_ipaddr),
                                      DEST_PORT, timeouts);

    // Printing out information that we are starting
    // a connection to the Socks5 proxy server
    println!("Starting connection to the socks5 proxy server `{}`", proxy_addr);

    // Connecting to the stream and getting the readable and
    // writable stream, or terminating the script if it is
    // unable to connect
    let stream = TcpStream::connect(proxy_addr)
                           .await
                           .expect("Unable to connect to the proxy server");


    // Printing out information that we are starting
    // a connection to the service through the proxy client
    println!("Starting connection to the destination `{}:{}` throught socks5 proxy `{}`",
              dest_ipaddr, DEST_PORT, proxy_addr);

    // Connecting to the service through proxy
    let stream = match socks5_proxy.connect(stream).await {
        Ok(stream) => {
            // Successfully connected to the service
            stream
        },
        Err(e) => {
            // -- handling error -- //
            exit(1);
        }
    };
    
    // -- using `stream` -- //
}
