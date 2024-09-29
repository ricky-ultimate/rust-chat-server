use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    // Bind a TCP listener on the specified address.
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server running on 127.0.0.1:8080");

    // Broadcast channel to send messages to all clients.
    let (tx, _rx) = broadcast::channel::<String>(10);

    loop {
        // Accept incoming connections.
        let (socket, addr) = listener.accept().await.unwrap();
        println!("Client connected: {}", addr);

        // Clone the transmitter for each client.
        let tx = tx.clone();
        // Each client should have its own receiver.
        let mut rx = tx.subscribe();

        // Spawn a new task to handle each client concurrently.
        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    // Read from the client.
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        let msg = format!("{}: {}", addr, line.trim());
                        tx.send(msg).unwrap();
                        line.clear();
                    }

                    // Receive a broadcasted message and write to the client.
                    msg = rx.recv() => {
                        let msg = msg.unwrap();
                        if writer.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                }
            }

            println!("Client disconnected: {}", addr);
        });
    }
}
