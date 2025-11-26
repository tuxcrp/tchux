use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
};

use crate::utils::{decrypt_message, generate_key, input};

mod tui;

pub fn client(addr: String, passphrase: String) -> anyhow::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    let key = generate_key(passphrase.as_str());

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap_or_else(|_| {
        println!("\x1B[31mFailed to read from the server\x1B[0m");
        std::process::exit(1);
    });

    let encrypted_handshake = String::from_utf8_lossy(&buffer[..bytes_read]);
    let handshake_bytes: &[u8] = encrypted_handshake.as_bytes();

    let decrypted_handshake = decrypt_message(&key, handshake_bytes, true);

    stream.write_all(decrypted_handshake.as_bytes()).unwrap();
    let name = input("Name");
    stream.write_all(name.as_bytes())?;

    let (tx, rx) = mpsc::channel::<tui::AppEvent>();

    let mut stream_clone = stream.try_clone()?;
    let tx_clone = tx.clone();

    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match stream_clone.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let mut message = String::from_utf8_lossy(&buf[..n]).to_string();
                    if message.starts_with('+') {
                        message = message.chars().skip(1).collect();
                    } else {
                        message = decrypt_message(&key[..], &buf[..n], false).to_string();
                    }
                    tx_clone.send(tui::AppEvent::NetMessage(message)).ok();
                }
                _ => {
                    tx_clone.send(tui::AppEvent::Quit).ok();
                    return;
                }
            }
        }
    });
    tui::tui(tx, rx, &key, stream)
}
