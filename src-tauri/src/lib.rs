// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::image::Image;
pub mod base;
pub mod db;
mod model;

use db::{ClipboardDatabase, ClipboardEntry};
use model::ClipboardEvent;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State, PhysicalSize, PhysicalPosition};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

// Application state to hold the database connection
struct AppState {
    db: Mutex<ClipboardDatabase>,
    last_tray_rect: Mutex<Option<tauri::Rect>>,
}

#[tauri::command]
fn get_clipboard_text() -> String {
    base::get_clipboard_text()
}

fn save_clipboard_event(
    state: State<AppState>,
    clipboardEvent: ClipboardEvent,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    match db.save_entry(clipboardEvent) {
        Ok(id) => {
            println!("Clipboard event saved with id: {:?}", id);
            Ok(id)
        }
        Err(e) => {
            println!("Error saving clipboard event: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_recent_clipboard_entries(
    limit: usize,
    state: State<AppState>,
) -> Result<Vec<ClipboardEntry>, String> {
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

#[tauri::command]
fn load_clipboard_events_at_startup(app_handle: AppHandle) -> Result<(), String> {
    println!("Loading clipboard events at startup");
    let state = app_handle.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let entries = db.get_all_entries().map_err(|e| e.to_string())?;
    for entry in entries {
        println!(
            "Emitting load_clipboard_events_at_startup event: {:?}",
            entry
        );
        app_handle.emit("clipboard-changed", entry.clone()).unwrap();
    }
    Ok(())
}

#[tauri::command]
fn hide_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

fn spawn_clipboard_polling_thread(app_handle: AppHandle) -> Result<(), String> {
    let mut last_event: Option<ClipboardEvent> = None;
    println!("Spawning clipboard polling thread");
    thread::spawn(move || loop {
        let text = get_clipboard_text();
        if let Some(ref last_event) = last_event {
            if last_event.text() == text {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        }
        last_event = Some(ClipboardEvent::new(text));
        app_handle
            .emit("clipboard-changed", last_event.clone().unwrap())
            .unwrap();
        save_clipboard_event(app_handle.state::<AppState>(), last_event.clone().unwrap()).unwrap();
        thread::sleep(Duration::from_secs(1));
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
        .manage(AppState {
            db: Mutex::new(db),
            last_tray_rect: Mutex::new(None),
        })
        .setup(|app| {
            println!("Loadindg tray icon..");

            let icon_bytes = include_bytes!("../icons/tray-icon.png");
            let icon = Image::from_bytes(icon_bytes)?;
            println!("Tray icon loaded successfully");


            let app_handle = app.handle().clone();
            spawn_clipboard_polling_thread(app_handle.clone())?;

            // Create tray icon with menu
            let open_item = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_item, &settings_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                // Use saved tray rect if available
                                if let Some(state) = app.try_state::<AppState>() {
                                    if let Ok(last_rect) = state.last_tray_rect.lock() {
                                        if let Some(rect) = *last_rect {
                                            let scale_factor = window.scale_factor().unwrap_or(1.0);
                                            let physical_pos: PhysicalPosition<i32> = rect.position.to_physical(scale_factor);
                                            let physical_size: PhysicalSize<i32> = rect.size.to_physical(scale_factor);

                                            let window_width = 400;
                                            let window_x = physical_pos.x + (physical_size.width as i32 / 2) - (window_width / 2);
                                            let window_y = physical_pos.y + physical_size.height as i32 + 5;

                                            let _ = window.set_position(tauri::Position::Physical(
                                                tauri::PhysicalPosition { x: window_x, y: window_y }
                                            ));
                                        }
                                    }
                                }
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
                    println!("Tray icon event: {:?}", event);
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
                                // Calculate position based on tray icon rect
                                let scale_factor = window.scale_factor().unwrap_or(1.0);
                                let physical_pos: PhysicalPosition<i32> = rect.position.to_physical(scale_factor);
                                let physical_size: PhysicalSize<i32> = rect.size.to_physical(scale_factor);

                                let window_width = 400;
                                let window_x = physical_pos.x + (physical_size.width as i32 / 2) - (window_width / 2);
                                let window_y = physical_pos.y + physical_size.height as i32 + 5;

                                println!("=== Tray Icon Position Debug ===");
                                println!("Scale factor: {}", scale_factor);
                                println!("Tray position: x={}, y={}",
                                physical_pos.x, physical_pos.y);
                                println!("Tray size: width={}, height={}",
                                physical_size.width, physical_size.height);

                                let _ = window.set_position(tauri::Position::Physical(
                                    tauri::PhysicalPosition { x: window_x, y: window_y }
                                ));
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
            get_clipboard_text,
            get_recent_clipboard_entries,
            delete_clipboard_entry,
            clear_clipboard_history,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
