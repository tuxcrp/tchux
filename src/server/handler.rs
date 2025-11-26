use std::sync::Arc;

use crate::utils::{broadcast_message, encrypt_message, generate_key, get_color, ClientMap};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub async fn handle_client(
    stream: TcpStream,
    clients: ClientMap,
    passphrase: String,
) -> anyhow::Result<()> {
    let (mut reader, writer) = stream.into_split();
    let writer = Arc::new(Mutex::new(writer));
    // Handshake
    let mut buffer = [0; 1024];

    let key = generate_key(passphrase.as_str());
    let msg = uuid::Uuid::new_v4().to_string();
    let encrypted_handshake = encrypt_message(&key[..], &msg);

    writer
        .lock()
        .await
        .write_all(encrypted_handshake.as_bytes())
        .await?;

    let bytes_read = reader.read(&mut buffer).await.unwrap();
    let client_res = String::from_utf8_lossy(&buffer[..bytes_read]);
    let client_res = client_res.trim();

    if client_res != msg {
        tracing::error!("Handshake mismatch [{client_res} != {msg}]");
        broadcast_message(
            &clients,
            "WARNING: Attempted login with invalid passphrase!",
            "server",
        )
        .await?;
        return Err(anyhow::anyhow!("Handshake Failure"));
    }

    // Get username
    let user_name = match reader.read(&mut buffer).await {
        Ok(n) if n > 0 => String::from_utf8_lossy(&buffer[0..n]).trim().to_string(),
        _ => return Err(anyhow::anyhow!("Read Failure.")),
    };

    if user_name == "server" {
        return Ok(());
    }

    // Insert client
    clients
        .lock()
        .await
        .insert(user_name.clone(), writer.clone());

    // Welcome message
    let color = get_color(&user_name);
    let welcome_message = format!("{}{}\x1B[0m just joined the chat :)", color, user_name);
    broadcast_message(&clients, &welcome_message, "server").await?;
    tracing::info!("New user connected: {}", user_name);

    // Recv loop
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[0..n]).trim().to_string();
                tracing::info!("Received from {}: {}", user_name, message);
                broadcast_message(&clients, &message, &user_name).await?;
            }
            Err(e) => {
                tracing::error!("Error reading from client: {}", e);
                break;
            }
        }
    }

    // Remove client
    clients.lock().await.remove(&user_name);

    // Farewell message
    let color = get_color(&user_name);
    let farewell_message = format!("{}{}\x1B[0m left :(", color, user_name);
    broadcast_message(&clients, &farewell_message, "server").await?;
    tracing::info!("User disconnected: {}", user_name);
    Ok(())
}
