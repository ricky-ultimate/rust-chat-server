use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use log::info;
use crate::utils::{add_message, SharedHistory};

pub struct Client {
    pub username: String,
    pub addr: SocketAddr,
    writer: OwnedWriteHalf,
    receiver: broadcast::Receiver<String>,
}

impl Client {
    pub fn new(addr: SocketAddr, writer: OwnedWriteHalf, receiver: broadcast::Receiver<String>) -> Self {
        Client {
            username: addr.to_string(),
            addr,
            writer,
            receiver,
        }
    }

    pub async fn handle_outgoing_messages(&mut self) {
        while let Ok(msg) = self.receiver.recv().await {
            if let Err(_) = self.writer.write_all(msg.as_bytes()).await {
                break;
            }
        }
    }

    pub async fn send_message(&mut self, msg: &str) -> Result<(), tokio::io::Error> {
        self.writer.write_all(msg.as_bytes()).await
    }
}

pub async fn handle_client(
    mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    mut client: Client,
    tx: broadcast::Sender<String>,
    clients: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    history: SharedHistory,
) {
    let mut line = String::new();

    // Read initial username from the client
    if let Ok(_) = reader.read_line(&mut line).await {
        client.username = line.trim().to_string();
        clients.lock().unwrap().insert(client.username.clone(), tx.clone());
        let welcome_msg = format!("{} joined the chat!", client.username);
        add_message(&history, welcome_msg.clone(), 50);
        tx.send(welcome_msg).unwrap();
        info!("{} set as username for {}", client.username, client.addr);
    }

    line.clear();
    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break;
                }

                if line.starts_with("/msg") {
                    if let Some((recipient, message)) = parse_private_message(&line) {
                        if let Some(recipient_tx) = clients.lock().unwrap().get(&recipient) {
                            let private_msg = format!("(Private) {}: {}", client.username, message);
                            recipient_tx.send(private_msg).unwrap();
                        }
                    }
                } else {
                    let msg = format!("{}: {}", client.username, line.trim());
                    add_message(&history, msg.clone(), 50);
                    if tx.send(msg).is_err() {
                        break;
                    }
                }

                line.clear();
            }

            _ = client.handle_outgoing_messages() => {}
        }
    }

    let goodbye_msg = format!("{} left the chat!", client.username);
    add_message(&history, goodbye_msg.clone(), 50);
    tx.send(goodbye_msg).unwrap();
    clients.lock().unwrap().remove(&client.username);
    info!("Client {} disconnected", client.username);
}

// Parses a private message command of the form "/msg recipient message".
fn parse_private_message(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        None
    } else {
        Some((parts[1].to_string(), parts[2].to_string()))
    }
}
