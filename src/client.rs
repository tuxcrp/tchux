use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn input(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut out = String::new();
    io::stdin().read_line(&mut out).unwrap();
    print!("\x1B[F\x1B[K");
    io::stdout().flush().unwrap();
    out
}

pub fn client(serveraddr: Option<&str>) {
    let server_address = {
        if serveraddr.is_none() {
            input("Server Address: ")
        } else {
            serveraddr.unwrap().to_string()
        }
    };

    let server_address = server_address.trim();
    let mut stream = TcpStream::connect(server_address).unwrap();
    println!("Connected to the server at {}", server_address);

    let name = input("Name: ");
    let name = name.trim();

    stream.write_all(name.as_bytes()).unwrap();

    let mut stream_clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match stream_clone.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let message = String::from_utf8_lossy(&buffer[..n]);
                    println!("{}", message);
                }
                _ => break,
            }
        }
    });

    loop {
        thread::sleep(Duration::from_millis(100));
        let message = input("> ");
        let message = message.trim();
        if message.len() > 0 {
            if message.chars().next().unwrap() == ':' {
                break;
            }
            stream.write_all(message.as_bytes()).unwrap();
        }
    }

    println!("Disconnected from the server.");
}
