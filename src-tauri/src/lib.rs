// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod base;
pub mod db;
mod model;

use db::{ClipboardDatabase, ClipboardEntry};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use chrono::Utc;
use tauri::{State, AppHandle, Emitter};
use model::ClipboardEvent;

// Application state to hold the database connection
struct AppState {
    db: Mutex<ClipboardDatabase>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_clipboard_text() -> String {
    base::get_clipboard_text()
}

#[tauri::command]
fn save_clipboard_entry(content: String, state: State<AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_entry(content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_clipboard_entries(state: State<AppState>) -> Result<Vec<ClipboardEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_entries().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recent_clipboard_entries(limit: usize, state: State<AppState>) -> Result<Vec<ClipboardEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_recent_entries(limit).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_clipboard_entry(id: i64, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_clipboard_history(state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_all().map_err(|e| e.to_string())
}

fn spawn_clipboard_polling_thread(app_handle: AppHandle) -> Result<(), String> {
    let mut last_text = String::new();
    println!("Spawning clipboard polling thread");
    thread::spawn(move || {
        loop {
            let text = get_clipboard_text();
                let event = ClipboardEvent::new(text);
                println!("Emitting clipboard-changed event: {:?}", event);
                app_handle.emit("clipboard-changed", event).map_err(|e| e.to_string())?;
            thread::sleep(Duration::from_secs(1));
        }
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO(human): Configure the database path
    // The database file will be stored in the app's data directory.
    // You should decide the best location based on your app's requirements.
    // Options include:
    // 1. App data directory: app.path().app_data_dir() - platform-specific app data
    // 2. App local data: app.path().app_local_data_dir() - local to machine
    // 3. Custom path: Define your own path
    // For now, using a default "clipboard_history.db" in current directory

    let db_path = std::path::PathBuf::from("clipboard_history.db");
    let db = ClipboardDatabase::new(db_path).expect("Failed to initialize database");

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            spawn_clipboard_polling_thread(app_handle)?;
            Ok(())
        })
        .manage(AppState { db: Mutex::new(db) })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_clipboard_text,
            save_clipboard_entry,
            get_all_clipboard_entries,
            get_recent_clipboard_entries,
            delete_clipboard_entry,
            clear_clipboard_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
