use aes_gcm::{
    aead::{rand_core::RngCore, Aead, OsRng},
    Aes256Gcm, KeyInit, Nonce,
};
use base64::{prelude::BASE64_URL_SAFE, Engine};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
    sync::Arc,
};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::Mutex};

const COLORS: [&str; 5] = ["\x1B[32m", "\x1B[33m", "\x1B[34m", "\x1B[35m", "\x1B[36m"];

pub type ClientMap = Arc<Mutex<HashMap<String, Arc<Mutex<OwnedWriteHalf>>>>>;

pub fn get_color(username: &str) -> &'static str {
    if username == "server" {
        return "+\x1B[31m";
    }
    let hash: u32 = username
        .bytes()
        .fold(0, |acc, b| acc.wrapping_add(b as u32));
    let index = hash as usize % COLORS.len();
    COLORS[index]
}

pub fn get_time() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

pub async fn broadcast_message(
    clients: &ClientMap,
    message: &str,
    sender: &str,
) -> anyhow::Result<()> {
    let clients = clients.lock().await;
    let sender_color = get_color(sender);

    for writer in clients.values() {
        let colored_message = format!(
            "{}❮{}❯ {}❯\x1B[0m: {}",
            sender_color,
            get_time(),
            sender,
            message
        );

        writer
            .lock()
            .await
            .write_all(colored_message.as_bytes())
            .await?
    }
    tracing::info!("Broadcast: {}", message);
    Ok(())
}

pub fn generate_key(passphrase: &str) -> [u8; 32] {
    let mut hasher = <Sha256 as Digest>::new();
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
    BASE64_URL_SAFE.encode(encrypted_message)
}
pub fn decrypt_message(key: &[u8], ciphertext: &[u8], handshake: bool) -> String {
    let decoded = String::from_utf8_lossy(ciphertext).to_string();
    let decoded_message = decoded.split(": ").last().unwrap();
    let sender = decoded.split(": ").next().unwrap();

    let ciphertext = BASE64_URL_SAFE.decode(decoded_message).unwrap_or_else(|_| {
        println!("Unable to decode the message. This should not happen!");
        std::process::exit(1);
    });

    let cipher = Aes256Gcm::new(key.into());

    // Extract the nonce (first 12 bytes)
    let nonce = Nonce::from_slice(&ciphertext[..12]);

    // Extract the actual ciphertext (after the nonce)
    let ciphertext = &ciphertext[12..];

    // Decrypt the message
    let decrypted = {
        match cipher.decrypt(nonce, ciphertext) {
            Ok(val) => val,
            _ => {
                println!("Wrong passphrase! This action will be reported!");
                std::process::exit(1);
            }
        }
    };
    // Anyone who knows the shared passphrase can encrypt arbitrary bytes, including
    // non-UTF-8 ones, so a peer could otherwise crash every other client on purpose.
    let decrypted_string = String::from_utf8_lossy(&decrypted).to_string();

    if handshake {
        decrypted_string
    } else {
        format!("{sender} {}", decrypted_string)
    }
}

pub fn input(prompt: &str) -> String {
    print!("\x1B[32m{prompt}❯ \x1b[0m");
    if stdout().flush().is_err() {
        eprintln!("tchux: failed to write to stdout");
        std::process::exit(1);
    }
    let mut out = String::new();
    if stdin().read_line(&mut out).is_err() {
        eprintln!("tchux: failed to read input");
        std::process::exit(1);
    }
    print!("\x1B[F\x1B[K");
    let _ = stdout().flush();
    out.trim().to_string()
}

pub fn emoji_preprocessor(_in: String) -> String {
    let emojis = HashMap::from([
        (":happy:".to_string(), "😊"),
        (":sad:".to_string(), "😢"),
        (":angry:".to_string(), "😠"),
        (":laughing:".to_string(), "😂"),
        (":heart:".to_string(), "❤️"),
        (":heartbroken:".to_string(), "💔"),
        (":thinking:".to_string(), "🤔"),
        (":sleeping:".to_string(), "😴"),
        (":winking:".to_string(), "😉"),
        (":surprised:".to_string(), "😲"),
        (":skull:".to_string(), "💀"),
        (":sparkle:".to_string(), "✨"),
    ]);
    let mut out = String::new();
    for token in _in.split(' ') {
        let mut is_emoji = false;
        for emoji_key in emojis.keys() {
            if token == emoji_key {
                out.push_str(emojis.get(emoji_key).unwrap().to_string().as_str());
                out.push(' ');
                is_emoji = true;
                break;
            }
        }
        if !is_emoji {
            out.push_str(token);
            out.push(' ');
        }
    }
    out.trim().to_string()
}
