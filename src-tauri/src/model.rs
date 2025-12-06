use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ClipboardEvent {
    text: String,
    timestamp: u64
}

impl ClipboardEvent {
    pub fn new(text: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self { text, timestamp }
    }
}