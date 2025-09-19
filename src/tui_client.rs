use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{
    io::{self, stdout, Read, Write},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

struct ChatState {
    messages: Vec<String>,
    input_buffer: String,
    connected: bool,
    name: String,
    stream: Option<TcpStream>,
    encryption_key: [u8; 32],
}

impl ChatState {
    fn new(key: [u8; 32]) -> Self {
        Self {
            messages: Vec::new(),
            input_buffer: String::new(),
            connected: false,
            name: String::new(),
            stream: None,
            encryption_key: key,
        }
    }

    fn add_message(&mut self, message: String) {
        self.messages.push(process(message));
    }
}

// Reuse your existing encryption functions
fn generate_key(passphrase: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..]);
    key
}

fn encrypt_message(key: &[u8], plaintext: &str) -> String {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encryption failure!");
    let encrypted_message = [nonce_bytes.to_vec(), ciphertext].concat();
    URL_SAFE.encode(encrypted_message)
}

fn decrypt_message(key: &[u8], ciphertext: &[u8], handshake: bool) -> String {
    let decoded = String::from_utf8_lossy(ciphertext).to_string();
    let decoded_message = decoded.split(": ").last().unwrap();
    let sender = decoded.split(": ").next().unwrap();

    let ciphertext = URL_SAFE.decode(decoded_message).unwrap();
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let ciphertext = &ciphertext[12..];
    let decrypted = match cipher.decrypt(nonce, ciphertext) {
        Ok(val) => val,
        _ => {
            return "Wrong passphrase! This action will be reported!".to_string();
        }
    };
    let decrypted_string = String::from_utf8(decrypted).unwrap();

    if handshake {
        decrypted_string
    } else {
        format!("{sender} {}", decrypted_string)
    }
}

fn process(input: String) -> String {
    use std::collections::HashMap;
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
    for token in input.split(' ') {
        let mut is_emoji = false;
        for emoji_key in emojis.keys() {
            if token == emoji_key {
                out.push_str(emojis.get(emoji_key).unwrap());
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

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn handle_connection(
    stream: &mut TcpStream,
    key: [u8; 32],
    name: &str,
    tx: Sender<String>,
) -> io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let encrypted_handshake = String::from_utf8_lossy(&buffer[..bytes_read]);
    let handshake_bytes: &[u8] = encrypted_handshake.as_bytes();
    let decrypted_handshake = decrypt_message(&key, handshake_bytes, true);

    if decrypted_handshake != "this is a cat >:) meow" {
        tx.send("Handshake failed. Disconnecting.".to_string())
            .unwrap();
        return Ok(());
    }

    stream.write_all(decrypted_handshake.as_bytes())?;
    stream.write_all(name.as_bytes())?;

    let mut stream_clone = stream.try_clone()?;
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match stream_clone.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let mut message = String::from_utf8_lossy(&buffer[..n]).to_string();
                    match message.chars().next().unwrap() {
                        '+' => message = message.chars().skip(1).collect(),
                        _ => message = decrypt_message(&key, &buffer[..n], false),
                    }
                    tx.send(message).unwrap();
                }
                _ => {
                    tx.send("Disconnected from server.".to_string()).unwrap();
                    break;
                }
            }
        }
    });

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut ChatState,
    rx: &Receiver<String>,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(frame.size());

            let messages: Vec<ListItem> = state
                .messages
                .iter()
                .map(|m| ListItem::new(m.as_str()))
                .collect();

            let messages =
                List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));

            let input = Paragraph::new(state.input_buffer.as_str())
                .block(Block::default().borders(Borders::ALL).title("Input"));

            frame.render_widget(messages, chunks[0]);
            frame.render_widget(input, chunks[1]);
        })?;

        // Check for new messages
        if let Ok(message) = rx.try_recv() {
            state.add_message(message);
            continue;
        }

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Enter => {
                        if !state.input_buffer.is_empty() {
                            if let Some(stream) = &mut state.stream {
                                if state.input_buffer.starts_with('\\') {
                                    match state.input_buffer.as_str() {
                                        "\\exit" => break,
                                        _ => (),
                                    }
                                } else {
                                    let encrypted =
                                        encrypt_message(&state.encryption_key, &state.input_buffer);
                                    stream.write_all(encrypted.as_bytes())?;
                                }
                            }
                            state.input_buffer.clear();
                        }
                    }
                    KeyCode::Char(c) => {
                        state.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        state.input_buffer.pop();
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

pub fn client(serveraddr: &str, name: &str, passphrase: &str) {
    let key = generate_key(passphrase);
    let mut state = ChatState::new(key);

    let mut terminal = setup_terminal().unwrap();

    let (tx, rx) = mpsc::channel();

    let mut stream = TcpStream::connect(serveraddr.trim()).unwrap();
    state.stream = Some(stream.try_clone().unwrap());
    state.connected = true;
    state.name = name.to_string();

    handle_connection(&mut stream, key, &state.name, tx).unwrap();

    run_app(&mut terminal, &mut state, &rx).unwrap();

    restore_terminal(&mut terminal).unwrap();
}
