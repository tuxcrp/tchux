use std::{collections::HashMap, fs::File, sync::Arc};

use crate::utils::ClientMap;
use tokio::{net::TcpListener, sync::Mutex};

mod handler;

pub async fn server(port: String, passphrase: String) -> anyhow::Result<()> {
    let file = File::create(format!("tchux-{port}.log")).unwrap();
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|err| {
        tracing::error!("Unable to bind to {addr}: {err}");
        println!("Unable to bind to {addr}: {err}");
        drop(_guard);
        std::process::exit(1);
    });

    tracing::info!("Tchux Server listening on {addr}");

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    ctrlc::set_handler(move || {
        print!("\r\x1B[K");
        tracing::info!("Server shutting down...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler.");

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
