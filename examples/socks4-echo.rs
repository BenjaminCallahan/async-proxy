use clap::{Arg, App};
use async_proxy::clients::socks4::general::{
    Socks4General, ConnParams
};
use async_proxy::general::ConnectionTimeouts;
use async_proxy::proxy::ProxyStream;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use std::time::Duration;
use ansi_term::Color;

/// Prints out beautiful script messages
macro_rules! message {
    // Prints out a success message
    (Success, $m:expr $(, $a:expr)* $(,)?) => {
        print!("{} ", Color::Green.bold().paint("Success:"));
        println!($m, $($a), *);
    };
    // Prints out an info message
    (Info, $m:expr $(, $a:expr)* $(,)?) => {
        print!("{} ", Color::White.bold().paint("Info:"));
        println!($m, $($a), *);
    };
    // Prints out an error message
    (Error, $m:expr $(, $a:expr)* $(,)?) => {
        print!("{} ", Color::Red.bold().paint("Error:"));
        println!($m, $($a), *);
    };
    // Not only prints out an error message,
    // but also terminates the script process
    (Fatal, $m:expr $(, $a:expr)* $(,)?) => {
        message!(Error, $m, $($a), *);
        // Terminating the script process
        std::process::exit(1);
    };
}

/// Trait which function `fatal`
/// throws out a fatal error if it
/// is unable to extract `ok` value and
/// terminates execution of the process 
trait Fatal<T> {
    fn fatal(self, message: &str) -> T;
} 

impl<T, E> Fatal<T> for Result<T, E> {
    fn fatal(self, message: &str) -> T {
        // Matching result and extracting
        // the value if the result is `ok`,
        // unless throwing a fatal error with
        // the given message
        match self {
            Ok(value) => value,
            Err(_) => {
                message!(Fatal, "{}", message);
            }
        }
    }
}

impl<T> Fatal<T> for Option<T> {
    fn fatal(self, message: &str) -> T {
        // Extracting a value if exists,
        // unless throwing a fatal error
        match self {
            Some(value) => value,
            None => {
                message!(Fatal, "{}", message);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Parsing the script arguments
    let matches = App::new("Example program using async proxies")
                      .version("0.1.0")
                      .author("TonyGraim")
                      // The actual Socks4-proxy server address
                      .arg(Arg::with_name("proxy-address")
                               .short("p")
                               .long("proxy-addr")
                               .takes_value(true)
                               .required(true)
                               .help("The address of the socks4-proxy in format `ip:port`"))
                      // The address of a service
                      // the script will be connecting to
                      // through proxy
                      .arg(Arg::with_name("destination")
                               .short("d")
                               .long("destination")
                               .takes_value(true)
                               .required(true)
                               .help("The destination point in format `ipv4:port`"))
                      // Ident. See Socks4 proxification protocol
                      // for more information
                      .arg(Arg::with_name("ident")
                               .short("i")
                               .long("ident")
                               .takes_value(true)
                               .default_value("")
                               .help("The ident used for Socks4 connection establishment"))
                      .get_matches();

    // Getting out the 'proxy-address' argument value
    let server_addr = matches.value_of("proxy-address").unwrap();
    // Getting out the 'destination' argument value
    let destination = matches.value_of("destination").unwrap();
    // Getting out the 'ident' argument value
    let ident = matches.value_of("ident").unwrap();

    // Setting up timeouts
    let timeouts = ConnectionTimeouts::new(
        // Connecting timeout
        Duration::from_secs(8),
        // Write timeout
        Duration::from_secs(8),
        // Read timeout
        Duration::from_secs(8)
    );

    // Shadowing ident value and converting it to a `Cow`
    let ident = std::borrow::Cow::Owned(ident.to_owned());
    
    // Creating required connection parameters
    // for Socks4 proxy client
    let connection_params = ConnParams::new(destination.parse().unwrap(),
                                            ident,
                                            timeouts);

    // Printing out information that we are starting
    // a connection to the Socks4 proxy server
    message!(Info, "Starting connection to the Socks4 proxy server `{}`", server_addr);

    // Extracting the server's `SocketAddr` from the
    // `server_addr`
    let server_socket_addr: SocketAddr = server_addr.parse().unwrap();

    // Connecting to the stream and getting the readable and
    // writable stream, or terminating the script if it is
    // unable to connect
    let stream = match TcpStream::connect(server_socket_addr).await {
        Ok(stream) => stream,
        Err(_) => {
            message!(Fatal, "Unable to connect to the proxy server `{}`", server_addr);
        }
    };

    // Printing out information that we are starting
    // a connection to the service through the proxy client
    message!(Info, "Starting connection to the destination `{}` throught socks4 proxy `{}`",
             destination, server_addr);

    // Connecting to the service through proxy
    let mut stream = match Socks4General::connect(stream, connection_params).await {
        Ok(stream) => {
            message!(Success, "Successfully connected to the service through the proxy");
            stream
        },
        Err(e) => {
            message!(Fatal, "Cannot connect to the service: {}", e);
        }
    };

    // Getting a message that will be sent to the service
    println!("Please inter a message to be sent.");
    print!("{} ", Color::White.bold().paint("Message:"));

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)
                    .fatal("Unable to read a line from stdin");

    // Sending the message to the service
    // with the timeout of 8 seconds
    let future = stream.write_all(input.as_bytes());
    let future = timeout(Duration::from_secs(8), future);
    future.await.fatal("Timeout of 8 seconds reached")
                .fatal("Unable to send the message");

    // Receiving a message from the service
    // with the timeout of 8 seconds
    let future = stream.read_to_string(&mut input);
    let future = timeout(Duration::from_secs(8), future);
    future.await.fatal("Timeout of 8 seconds reached")
                .fatal("Unable to receive a string from the service");

    message!(Success, "Received message from the service: {}", input);
    
}
