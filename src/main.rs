use std::{env::args, thread::sleep, time::Duration};

use crate::utils::input;

mod client;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    if let Some(mode) = args().nth(1) {
        match mode.as_str() {
            "server" => {
                let port = input("Enter port");
                let passphrase = input("Set passphrase");

                {
                    let port = port.clone();
                    let passphrase = passphrase.clone();
                    tokio::spawn(async move {
                        match server::server(port, passphrase).await {
                            Err(e) => println!("Server Error: \x1b[31m{}\x1b[0m", e),
                            _ => (),
                        }
                    });
                }
                sleep(Duration::from_millis(100));

                if let Err(e) = client::client(format!("0.0.0.0:{port}"), passphrase) {
                    println!("Client Error: \x1b[31m{}\x1b[0m", e);
                }
            }
            "client" => {
                let addr = input("Enter address:port");
                let passphrase = input("Enter passphrase");
                match client::client(addr, passphrase) {
                    Err(e) => println!("Error: \x1b[31m{}\x1b[0m", e),
                    _ => (),
                }
            }
            _ => println!("Invalid mode, `client` or `server`!"),
        }
    } else {
        println!("Arg 1 should specify mode, `client` or `server`!");
    }
}
