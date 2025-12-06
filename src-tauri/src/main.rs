// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clipboardwatcher_lib::db;
use clipboardwatcher_lib::base;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;


fn polling_clipboard(interval: Duration) {
    loop {
        println!("Before get_clipboard_text");
        let text = base::get_clipboard_text();
        println!("After get_clipboard_text {} ", text);
        thread::sleep(interval);
    }
}

fn main() {
    clipboardwatcher_lib::run()
}
