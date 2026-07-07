use std::{collections::HashMap, fs::File, sync::Arc};

use crate::utils::ClientMap;
use tokio::{net::TcpListener, sync::Mutex};

mod handler;

pub async fn server(port: String, passphrase: String) -> anyhow::Result<()> {
    let log_path = format!("tchux-{port}.log");
    let file = File::create(&log_path).unwrap_or_else(|err| {
        eprintln!("\x1b[31mUnable to create log file {log_path}: {err}\x1b[0m");
        std::process::exit(1);
    });
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|err| {
        tracing::error!("Unable to bind to {addr}: {err}");
        eprintln!("\x1b[31mUnable to bind to {addr}: {err}\x1b[0m");
        drop(_guard);
        std::process::exit(1);
    });

    tracing::info!("Tchux Server listening on {addr}");

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    if let Err(e) = ctrlc::set_handler(move || {
        print!("\r\x1B[K");
        tracing::info!("Server shutting down...");
        std::process::exit(0);
    }) {
        tracing::error!("Error setting Ctrl-C handler: {e}");
        eprintln!("\x1b[31mError setting Ctrl-C handler: {e}\x1b[0m");
        std::process::exit(1);
    }

    loop {
        let (stream, addr) = listener.accept().await?;
        let clients_clone = Arc::clone(&clients);
        let passphrase_clone = passphrase.clone();
        tokio::spawn(async move {
            if let Err(err) = handler::handle_client(stream, clients_clone, passphrase_clone).await
            {
                tracing::error!("Client at {addr} errored: {err}")
            }
        });
    }
}
