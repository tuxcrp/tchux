use simplelog::*;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use uuid::Uuid;

fn handle_client(mut stream: TcpStream) {
    let uuid_name = format!("tchux-{}.log", Uuid::new_v4());
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(uuid_name).unwrap(),
    )
    .unwrap();
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(0) => return,
        Ok(n) => {
            if n > 10 {}
            if stream
                .write_all(
                    format!("blue|Welcome {}", String::from_utf8_lossy(&buffer[0..n])).as_bytes(),
                )
                .is_err()
            {
                eprintln!("Error sending message, Closing Connection");
            }
        }
        Err(_) => return,
    }
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                log::info!("Recived: {}", String::from_utf8_lossy(&buffer[0..n]));
                // Echo back the received data
                if stream.write_all(&buffer[..n]).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

pub fn server(port: i16) {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).expect(&format!("unable to connect to [...]::{port}"));
    log::info!("Server listening on {addr}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("New Connection");
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
