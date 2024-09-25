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
            println!("Usage: tchux <server|client> [<port (default: 8080) passphrase |addr (for client) passphrase>]");
            exit(1);
        }
    };

    match mode.as_str() {
        "server" => {
            let port = match args.get(2) {
                Some(val) => val.parse::<i16>().unwrap(),
                None => 8080,
            };
            let passphrase = match args.get(3) {
                Some(val) => val.to_string(),
                None => "IWasSoDumbIDidNotSetAPassword".to_string(),
            };

            let passphrase_clone = passphrase.clone();
            thread::spawn(move || server::server(port, passphrase_clone));
            sleep(Duration::from_millis(200));
            client::client(format!("127.0.0.1:{port}").as_str(), &passphrase);
        }
        "client" => {
            let addr = match args.get(2) {
                Some(val) => val.clone(),
                None => {
                    println!(
                        "Usage: tchux <server|client> [<port (default: 8080) passphrase |addr (for client) passphrase>]"
                    );
                    exit(1);
                }
            };
            let passphrase = match args.get(3) {
                Some(val) => val.as_str(),
                None => "IWasSoDumbIDidNotSetAPassword",
            };

            client::client(addr.as_str(), passphrase);
        }
        _ => (),
    }
}
