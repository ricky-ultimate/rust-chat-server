mod client;
mod server;
mod utils;

use tokio::signal;
use server::run_server;
use utils::init_logging;

#[tokio::main]
async fn main() {
    init_logging();

    let server_handle = tokio::spawn(async {
        run_server("127.0.0.1:8080", 50).await.unwrap();
    });

    signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
    println!("Shutting down server...");
    server_handle.await.unwrap();
}
