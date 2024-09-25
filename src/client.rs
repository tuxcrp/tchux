use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::{env, thread};

pub fn generate_key(passphrase: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..]);
    key
}

pub fn encrypt_message(key: &[u8], plaintext: &str) -> String {
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

fn decrypt_message(key: &[u8], ciphertext: &[u8], handshake: bool) -> String {
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
    let decrypted_string = String::from_utf8(decrypted).unwrap();

    if handshake {
        decrypted_string
    } else {
        format!("{sender}: {}", decrypted_string)
    }
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

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).unwrap();

    let encrypted_handshake = String::from_utf8_lossy(&buffer[..bytes_read]);
    let handshake_bytes: &[u8] = encrypted_handshake.as_bytes();

    let decrypted_handshake = decrypt_message(&key, handshake_bytes, true);
    if decrypted_handshake != "this is a cat >:) meow" {
        println!("Handshake failed. Disconnecting.");
        return;
    }
    stream.write_all(decrypted_handshake.as_bytes()).unwrap();

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
                        _ => message = decrypt_message(&key[..], &buffer[..n], false).to_string(),
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
                '\\' => handle_commands(message),
                _ => {
                    let encrypted = encrypt_message(&key[..], message);
                    stream.write_all(encrypted.as_bytes()).unwrap();
                }
            }
        }
    }
}

fn handle_commands(command: &str) {
    match command {
        "\\exit" => {
            println!("\nDisconnected from the server.");
            std::process::exit(0);
        }
        "\\p" => {
            let home = env::var("HOME").unwrap();
            let shell = env::var("SHELL").unwrap();

            let history_file = if shell.ends_with("zsh") {
                PathBuf::from(&home).join(".zsh_history")
            } else {
                PathBuf::from(&home).join(".bash_history")
            };

            let file = File::open(&history_file).unwrap();
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

            let new_content: Vec<String> = lines
                .clone()
                .into_iter()
                .take(lines.len().saturating_sub(10))
                .collect();

            let mut file = File::create(history_file).unwrap();
            for line in new_content {
                writeln!(file, "{}", line).unwrap();
            }

            if cfg!(target_os = "windows") {
                Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
            } else {
                Command::new("clear").status().unwrap();
            }

            std::process::exit(0);
        }

        _ => (),
    }
}
