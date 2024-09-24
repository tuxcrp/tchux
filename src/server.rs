use simplelog::*;
use std::collections::HashMap;
use std::fs::{remove_file, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type ClientMap = Arc<Mutex<HashMap<String, (TcpStream, String)>>>;

const COLORS: [&str; 5] = ["\x1B[32m", "\x1B[33m", "\x1B[34m", "\x1B[35m", "\x1B[36m"];

// I give up, I can't read this shit - Mav
fn get_color(username: &str) -> &'static str {
    let hash: u32 = username
        .bytes()
        .fold(0, |acc, b| acc.wrapping_add(b as u32));
    let index = hash as usize % COLORS.len();
    COLORS[index]
}

fn handle_client(stream: TcpStream, clients: ClientMap) {
    let mut buffer = [0; 1024];

    #[allow(unused_assignments)]
    let mut user_name = String::new();

    match stream.try_clone().unwrap().read(&mut buffer) {
        Ok(n) if n > 0 => {
            user_name = String::from_utf8_lossy(&buffer[0..n]).trim().to_string();
        }
        _ => return,
    }

    clients.lock().unwrap().insert(
        user_name.clone(),
        (
            stream.try_clone().unwrap(),
            get_color(&user_name).to_string(),
        ),
    );

    {
        // welcome_message
        let color = get_color(&user_name);
        let welcome_message = format!("{}{}\x1B[0m just joined the chat :)", color, user_name);
        broadcast_message(&clients, &welcome_message, "server");
        log::info!("New user connected: {}", user_name);
        log::info!("Sent: {}", welcome_message);
    }

    loop {
        match stream.try_clone().unwrap().read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[0..n]).trim().to_string();
                log::info!("Received from {}: {}", user_name, message);

                broadcast_message(&clients, &message, &user_name);
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }

    clients.lock().unwrap().remove(&user_name);
    {
        // farewell_message
        let color = get_color(&user_name);
        let farewell_message = format!("{}{}\x1B[0m left :(", color, user_name);
        broadcast_message(&clients, &farewell_message, "server");
        log::info!("User disconnected: {}", user_name);
        log::info!("Sent: {}", farewell_message);
    }
}

fn broadcast_message(clients: &ClientMap, message: &str, sender: &str) {
    let clients = clients.lock().unwrap();
    let sender_color = clients
        .get(sender)
        .map(|(_, color)| color.as_str())
        .unwrap_or("+\x1B[31m");

    for (username, (stream, _)) in clients.iter() {
        let colored_message = format!("{}{}\x1B[0m: {}", sender_color, sender, message);

        if let Err(e) = stream
            .try_clone()
            .unwrap()
            .write_all(colored_message.as_bytes())
        {
            eprintln!("Error sending message to {}: {}", username, e);
        }
    }
    log::info!("Broadcast: {}", message);
}

pub fn server(port: i16) {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).expect(&format!("unable to connect to [...]::{port}"));
    log::info!("Server listening on {addr}");

    let log_name = format!("tchux-{port}.log");
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(&log_name).unwrap(),
    )
    .unwrap();
    log::info!("Logging to file: {}", log_name);

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    let clients_clone = Arc::clone(&clients);
    let log_name_clone = log_name.clone();
    ctrlc::set_handler(move || {
        println!("Server shutting down...");
        let mut clients = clients_clone.lock().unwrap();
        for (_, (stream, _)) in clients.iter_mut() {
            let _ = stream.write_all(b"Server shutting down. Goodbye!");
        }
        clients.clear();
        if let Err(e) = remove_file(&log_name_clone) {
            eprintln!("Error removing log file: {}", e);
        }
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients_clone = Arc::clone(&clients);
                thread::spawn(move || handle_client(stream, clients_clone));
            }
            Err(e) => {
                log::error!("Connection failed: {}", e);
            }
        }
    }
}
