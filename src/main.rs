mod client;
mod server;

use tokio::signal;
use server::run_server;

#[tokio::main]
async fn main() {
    let server_handle = tokio::spawn(async {
        run_server("127.0.0.1:8080").await.unwrap();
    });

    signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
    println!("Shutting down server...");
    server_handle.await.unwrap();
}
