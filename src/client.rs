use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;

fn generate_key(passphrase: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..]);
    key
}

fn encrypt_message(key: &[u8], plaintext: &str) -> String {
    let cipher = Aes256Gcm::new(key.into());

    // Generate a random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the message
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encryption failure!");

    // Concatenate nonce and ciphertext
    let encrypted_message = [nonce_bytes.to_vec(), ciphertext].concat();

    // Encode the result in Base64 for string representation
    URL_SAFE.encode(encrypted_message)
}

fn decrypt_message(key: &[u8], ciphertext: &[u8]) -> String {
    let decoded = String::from_utf8_lossy(ciphertext).to_string();
    let decoded_message = decoded.split(": ").last().unwrap();
    let sender = decoded.split(": ").next().unwrap();

    let ciphertext = URL_SAFE.decode(decoded_message).unwrap();
    let cipher = Aes256Gcm::new(key.into());

    // Extract the nonce (first 12 bytes)
    let nonce = Nonce::from_slice(&ciphertext[..12]);

    // Extract the actual ciphertext (after the nonce)
    let ciphertext = &ciphertext[12..];

    // Decrypt the message
    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .expect("decryption failure!");

    format!("{sender}: {}", String::from_utf8(decrypted).unwrap())
}

fn input(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut out = String::new();
    io::stdin().read_line(&mut out).unwrap();
    print!("\x1B[F\x1B[K");
    io::stdout().flush().unwrap();
    out
}

pub fn client(serveraddr: &str, passphrase: &str) {
    let server_address = serveraddr.trim();
    let mut stream = TcpStream::connect(server_address).unwrap();
    println!("Connected to the server at {}", server_address);
    let key = generate_key(passphrase);

    let name = input("Name: ");
    let name = name.trim();

    stream.write_all(name.as_bytes()).unwrap();

    let mut stream_clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match stream_clone.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let mut message = String::from_utf8_lossy(&buffer[..n]).to_string();
                    match message.chars().next().unwrap() {
                        '+' => message = message.chars().skip(1).collect(),
                        _ => message = decrypt_message(&key[..], &buffer[..n]).to_string(),
                    }
                    // Clear the current line and move cursor to the beginning
                    print!("\r\x1B[K");
                    // Print the colored message
                    print!("{}", message);
                    // Reset color and print the input prompt again
                    print!("\x1B[0m\n> ");
                    io::stdout().flush().unwrap();
                }
                _ => {
                    println!("\nDisconnected from the server.");
                    std::process::exit(0);
                }
            }
        }
    });

    loop {
        let message = input("> ");
        let message = message.trim();
        if message.len() > 0 {
            match message.chars().next().unwrap() {
                ':' => {
                    println!(
                        "Commands have not yet been implemented, assuming you wanted to :exit"
                    );
                    break;
                }
                _ => {
                    let encrypted = encrypt_message(&key[..], message);
                    stream.write_all(encrypted.as_bytes()).unwrap();
                }
            }
        }
    }

    println!("Disconnected from the server.");
}
