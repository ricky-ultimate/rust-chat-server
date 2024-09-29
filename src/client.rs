use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::broadcast;
use std::net::SocketAddr;

pub struct Client {
    pub username: String,
    pub addr: SocketAddr,
    writer: OwnedWriteHalf,
    receiver: broadcast::Receiver<String>,
}

impl Client {
    pub fn new(addr: SocketAddr, writer: OwnedWriteHalf, receiver: broadcast::Receiver<String>) -> Self {
        Client {
            username: addr.to_string(),  // Default username is the IP address
            addr,
            writer,
            receiver,
        }
    }

    // Listen for messages sent to the client
    pub async fn handle_outgoing_messages(&mut self) {
        while let Ok(msg) = self.receiver.recv().await {
            if let Err(_) = self.writer.write_all(msg.as_bytes()).await {
                break;
            }
        }
    }
}

pub async fn handle_client(mut reader: BufReader<tokio::net::tcp::OwnedReadHalf>, mut client: Client, tx: broadcast::Sender<String>) {
    let mut line = String::new();

    // Read initial username from the client
    if let Ok(_) = reader.read_line(&mut line).await {
        client.username = line.trim().to_string();
        let welcome_msg = format!("{} joined the chat!", client.username);
        tx.send(welcome_msg).unwrap();
    }

    line.clear();
    loop {
        tokio::select! {
            // Handle incoming messages from the client
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break;
                }
                let msg = format!("{}: {}", client.username, line.trim());
                if tx.send(msg).is_err() {
                    break;
                }
                line.clear();
            }

            // Handle outgoing messages to the client
            _ = client.handle_outgoing_messages() => {}
        }
    }

    let goodbye_msg = format!("{} left the chat!", client.username);
    tx.send(goodbye_msg).unwrap();
}
