use async_proxy::clients::socks5::{no_auth::TcpNoAuth, Destination};
use async_proxy::general::ConnectionTimeouts;
use async_proxy::proxy::ProxyConstructor;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::exit;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

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
        Duration::from_secs(8),
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
    let mut socks5_proxy = TcpNoAuth::new(Destination::Ipv4Addr(dest_ipaddr), DEST_PORT, timeouts);

    // You can use socks5_proxy.with_authentication() for select `Username/Password` auth method 
    // socks5_proxy.with_authentication(username, password);

    // Printing out information that we are starting
    // a connection to the Socks5 proxy server
    println!(
        "Starting connection to the socks5 proxy server `{}`",
        proxy_addr
    );

    // Connecting to the stream and getting the readable and
    // writable stream, or terminating the script if it is
    // unable to connect
    let stream = TcpStream::connect(proxy_addr)
        .await
        .expect("Unable to connect to the proxy server");

    // Printing out information that we are starting
    // a connection to the service through the proxy client
    println!(
        "Starting connection to the destination `{}:{}` throught socks5 proxy `{}`",
        dest_ipaddr, DEST_PORT, proxy_addr
    );

    // Connecting to the service through proxy
    let mut stream = match socks5_proxy.connect(stream).await {
        Ok(stream) => {
            println!("Successfully connected to the service through the proxy");
            stream
        }
        Err(e) => {
            println!("Cannot connect to the service: {}", e);
            exit(1);
        }
    };

    // Getting a message that will be sent to the service
    println!("Please inter a message to be sent. Message: ");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Unable to read a line from stdin");

    // Sending the message to the service
    // with the timeout of 8 seconds
    let future = stream.write_all(input.as_bytes());
    let future = timeout(Duration::from_secs(8), future);
    future
        .await
        .expect("Timeout of 8 seconds reached")
        .expect("Unable to send the message");

    // Receiving a message from the service
    // with the timeout of 8 seconds
    let future = stream.read_to_string(&mut input);
    let future = timeout(Duration::from_secs(8), future);
    future
        .await
        .expect("Timeout of 8 seconds reached")
        .expect("Unable to receive a string from the service");

    // Successfully received a message.
    // Printing it out
    println!("Received message from the service: {}", input);
}
