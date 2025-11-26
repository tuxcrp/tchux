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
                        server::server(port, passphrase)
                            .await
                            .expect("Server Panicked!")
                    });
                }
                sleep(Duration::from_millis(100));

                client::client(format!("0.0.0.0:{port}"), passphrase).expect("Client Panicked!");
            }
            "client" => {
                let addr = input("Enter address");
                let passphrase = input("Enter passphrase");
                client::client(addr, passphrase).expect("Client Panicked!");
            }
            _ => println!("Invalid mode, `client` or `server`!"),
        }
    } else {
        println!("Arg 1 should specify mode, `client` or `server`!");
    }
}
