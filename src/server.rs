use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::io::BufReader;
use crate::client::{Client, handle_client};

pub async fn run_server(addr: &str) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on {}", addr);

    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        let (reader, writer) = socket.into_split();
        let reader = BufReader::new(reader);
        let client = Client::new(addr, writer, rx);

        tokio::spawn(async move {
            handle_client(reader, client, tx).await;
        });
    }
}
