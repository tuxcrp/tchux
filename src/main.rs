use std::{
    env::args,
    process::exit,
    thread::{self, sleep},
    time::Duration,
};

mod client;
mod server;

fn main() {
    let mode = {
        match args().skip(1).next() {
            Some(val) => val,
            None => {
                println!("tchux <server|client>");
                exit(1)
            }
        }
    };

    match mode.as_str() {
        "server" => {
            thread::spawn(|| server::server());
            sleep(Duration::from_millis(200));
            client::client(Some("127.0.0.1:12345"));
        }
        "client" => client::client(None),
        _ => (),
    }
}
