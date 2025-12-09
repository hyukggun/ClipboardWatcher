use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::ClipboardEntry;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialEq)]
pub struct ClipboardEvent {
    text: String,
    timestamp: u64
}

impl ClipboardEvent {
    pub fn new(text: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self { text, timestamp }
    }

    pub fn from_entry(entry: ClipboardEntry) -> Self {
        Self { text: entry.content, timestamp: entry.created_at.parse::<u64>().unwrap() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}