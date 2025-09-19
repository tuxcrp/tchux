use std::{
    env::args,
    io::Write,
    process::exit,
    thread::{self, sleep},
    time::Duration,
};

mod client;
mod server;
mod tui_client;

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
        "server+tui" => {
            println!("Tui is not yet implemented");
        }
        "client+tui" => {
            print!("Enter your name: ");
            std::io::stdout().flush().unwrap();
            let name = {
                let mut name = String::new();
                std::io::stdin().read_line(&mut name).unwrap();
                name.trim().to_string()
            };

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

            tui_client::client(addr.as_str(), name.as_str(), passphrase);
        }
        _ => (),
    }
}
