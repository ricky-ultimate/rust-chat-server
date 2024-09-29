use log::{LevelFilter, info};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// A shared message history using Arc<Mutex<...>> for thread-safe access
pub type SharedHistory = Arc<Mutex<VecDeque<String>>>;

// Initialize the logger
pub fn init_logging() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)  // Capture logs at the INFO level or higher
        .init();
}

// Add a new message to the shared message history
pub fn add_message(history: &SharedHistory, message: String, limit: usize) {
    let mut history = history.lock().unwrap();
    if history.len() >= limit {
        history.pop_front();
    }
    history.push_back(message);
    info!("Message added to history: {}", message);
}

// Retrieve the recent message history for a new client
pub fn get_recent_history(history: &SharedHistory) -> Vec<String> {
    let history = history.lock().unwrap();
    history.iter().cloned().collect()
}
