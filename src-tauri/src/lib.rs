// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::image::Image;
pub mod base;
pub mod db;
mod model;
mod fzf;

use db::{ClipboardDatabase, ClipboardEntry};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use base::{get_current_clipboard_count, get_clipboard_text, get_clipboard_image};

// Application state to hold the database connection
struct AppState {
    db: Mutex<ClipboardDatabase>,
    last_tray_rect: Mutex<Option<tauri::Rect>>,
}

fn save_clipboard_event(
    state: State<AppState>,
    clipboard_entry: ClipboardEntry,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = db.save_entry(clipboard_entry.clone()).map_err(|e| e.to_string())?;
    println!("Clipboard entry saved with id: {:?}", id);
    Ok(id)
}

#[tauri::command]
fn delete_clipboard_entry(id: i64, state: State<AppState>, app_handle: AppHandle) -> Result<i64, String> {
    println!("Deleting clipboard entry with id: {:?}", id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let deleted_id = db.delete_entry(id).map_err(|e| e.to_string())?;
    println!("Clipboard entry deleted with id: {:?}", id);
    app_handle.emit("clipboard-deleted", deleted_id).map_err(|e| e.to_string())?;
    Ok(deleted_id)
}

#[tauri::command]
fn load_clipboard_events_at_startup(state: State<AppState>) -> Result<Vec<ClipboardEntry>, String> {
    println!("Loading clipboard events at startup");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let entries = db.get_all_entries().map_err(|e| e.to_string())?;
    println!("Loaded {} entries from database", entries.len());
    for entry in &entries {
        println!("  Entry: {:?}", entry);
    }
    Ok(entries)
}

#[tauri::command]
fn hide_window(app_handle: AppHandle) -> Result<(), String> {
    let window = app_handle.get_webview_window("main").ok_or("Main window not found".to_string())?;
    window.hide().map_err(|e| e.to_string())?;
    println!("Window hidden successfully");
    Ok(())
}

fn spawn_clipboard_polling_thread(app_handle: AppHandle) -> Result<(), String> {
    let mut current_count = 0;
    println!("[POLLING] Spawning clipboard polling thread");
    thread::spawn(move || loop {
        let new_count = get_current_clipboard_count();

        if new_count == current_count {
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        println!("[POLLING] Clipboard count changed: {} -> {}", current_count, new_count);
        current_count = new_count.clone();

        let mut entry = if let Some(text) = get_clipboard_text() {
            println!("[POLLING] Detected text entry");
            ClipboardEntry::new_text_entry(text)
        } else if let Some(image_path) = get_clipboard_image(new_count) {
            println!("[POLLING] Detected image entry");
            ClipboardEntry::new_image_entry(image_path)
        } else {
            println!("[POLLING] No text or image detected, skipping");
            thread::sleep(Duration::from_secs(1));
            continue;
        };


        match save_clipboard_event(app_handle.state::<AppState>(), entry.clone()) {
            Ok(id) => {
                println!("[POLLING] Entry saved with id: {}", id);
                entry.id = Some(id);
            }
            Err(e) => {
                println!("[POLLING] Error saving clipboard event: {:?}", e);
            }
        }

        println!("[POLLING] Emitting clipboard-changed event with id: {:?}", entry.id);
        // 프론트엔드로 이벤트 emit
        app_handle.emit("clipboard-changed", entry).unwrap();

        thread::sleep(Duration::from_secs(1));
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database in app data directory
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");

            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            let db_path = app_data_dir.join("clipboard_history.db");
            println!("Database path: {:?}", db_path);

            let db = ClipboardDatabase::new(db_path)
                .expect("Failed to initialize database");

            // Create and register AppState
            app.manage(AppState {
                db: Mutex::new(db),
                last_tray_rect: Mutex::new(None),
            });
            let icon_bytes = include_bytes!("../icons/icon32_32.png");
            let icon = Image::from_bytes(icon_bytes)?;

            let app_handle = app.handle().clone();
            spawn_clipboard_polling_thread(app_handle.clone())?;

            // Create tray icon with menu
            let open_item = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &settings_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                // Center window on screen
                                let _ = window.center();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "settings" => {
                            // Check if settings window already exists
                            if let Some(settings_window) = app.get_webview_window("settings") {
                                let _ = settings_window.show();
                                let _ = settings_window.set_focus();
                            } else {
                                // Create new settings window
                                use tauri::WebviewWindowBuilder;
                                let _ = WebviewWindowBuilder::new(
                                    app,
                                    "settings",
                                    tauri::WebviewUrl::App("/settings".into())
                                )
                                .title("Settings")
                                .inner_size(600.0, 400.0)
                                .resizable(true)
                                .build();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // println!("Tray icon event: {:?}", event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Down,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();

                        // Save tray icon rect for future use
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(mut last_rect) = state.last_tray_rect.lock() {
                                *last_rect = Some(rect);
                            }
                        }

                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                // Center window on screen
                                let _ = window.center();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                if !focused {
                    // Hide window when it loses focus
                    let _ = window.hide();
                }
            }
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_clipboard_events_at_startup,
            delete_clipboard_entry,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
