use std::{
    env::args,
    process::exit,
    thread::{self, sleep},
    time::Duration,
};

mod client;
mod server;

fn main() {
    let args: Vec<String> = args().collect();

    let mode = match args.get(1) {
        Some(val) => val.clone(),
        None => {
            println!("Usage: tchux <server|client> [<port (default: 8080)|addr (for client)>]");
            exit(1);
        }
    };

    match mode.as_str() {
        "server" => {
            let port = match args.get(2) {
                Some(val) => val.parse::<i16>().unwrap(),
                None => 8080,
            };

            thread::spawn(move || server::server(port));
            sleep(Duration::from_millis(200));
            client::client(Some(&format!("127.0.0.1:{port}")));
        }
        "client" => {
            let addr = match args.get(2) {
                Some(val) => val.clone(),
                None => {
                    println!(
                        "Usage: tchux <server|client> [<port (default: 8080)|addr (for client)>]"
                    );
                    exit(1);
                }
            };

            client::client(Some(&addr));
        }
        _ => (),
    }
}
