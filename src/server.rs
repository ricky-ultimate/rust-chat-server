use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::io::BufReader;
use crate::client::{Client, handle_client};
use crate::utils::{SharedHistory, add_message, get_recent_history};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::info;

pub async fn run_server(addr: &str, history_limit: usize) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("Server running on {}", addr);

    let (tx, _rx) = broadcast::channel(10);
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let history = Arc::new(Mutex::new(VecDeque::with_capacity(history_limit)));

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("Client connected: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let clients = Arc::clone(&clients);
        let history = Arc::clone(&history);

        let (reader, writer) = socket.into_split();
        let reader = BufReader::new(reader);
        let client = Client::new(addr, writer, rx);

        // Send recent message history to the new client
        let initial_history = get_recent_history(&history);
        tokio::spawn(async move {
            for msg in initial_history {
                let _ = client.writer.write_all(msg.as_bytes()).await;
            }
            handle_client(reader, client, tx, clients, history).await;
        });
    }
}
