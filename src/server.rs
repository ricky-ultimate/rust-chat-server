use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::io::BufReader;
use crate::client::{Client, handle_client};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub async fn run_server(addr: &str) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on {}", addr);

    // Create the broadcast channel for message communication and a shared client list.
    let (tx, _rx) = broadcast::channel(10);
    let clients = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // Accept an incoming client connection.
        let (socket, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);

        // Clone the necessary variables for each client.
        let tx = tx.clone();
        let rx = tx.subscribe();
        let clients = Arc::clone(&clients);

        // Split the socket into reader and writer.
        let (reader, writer) = socket.into_split();
        let reader = BufReader::new(reader);

        // Create a new client instance.
        let client = Client::new(addr, writer, rx);

        // Pass `clients` as the fourth argument.
        tokio::spawn(async move {
            handle_client(reader, client, tx, clients).await;
        });
    }
}
