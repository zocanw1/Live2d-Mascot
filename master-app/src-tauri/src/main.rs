mod commands;
mod tray;
mod overlay_manager;

use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub struct AppState {
    pub overlay_manager: Mutex<Option<overlay_manager::OverlayManager>>,
    pub main_window: Mutex<Option<WebviewWindow>>,
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC in master: {}", info);
        if let Some(dir) = dirs::data_dir() {
            let path = dir.join("live2d-mascot").join("master-crash.log");
            let _ = std::fs::create_dir_all(&path);
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
            {
                let _ = writeln!(f, "{}", msg);
            }
        }
    }));
}

fn setup_webview_folder() {
    let base = std::env::temp_dir().join("live2d-mascot-master-webview");
    let _ = std::fs::create_dir_all(&base);
    let current = format!("run-{}", std::process::id());
    if let Ok(entries) = std::fs::read_dir(&base) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("run-") && name != current {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
    let dir = base.join(&current);
    let _ = std::fs::create_dir_all(&dir);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &dir);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            overlay_manager: Mutex::new(None),
            main_window: Mutex::new(None),
        })
        .manage(commands::SettingsState(Mutex::new(
            commands::AppSettings::default(),
        )))
        .register_uri_scheme_protocol("live2d", |_app, request| {
            let path = request.uri().path();
            let decoded = percent_encoding::percent_decode_str(path)
                .decode_utf8_lossy()
                .to_string();
            let trimmed = decoded.trim_start_matches('/');
            let file_path = std::path::Path::new(trimmed);

            if let Ok(bytes) = std::fs::read(file_path) {
                let mime = if trimmed.ends_with(".json") {
                    "application/json"
                } else if trimmed.ends_with(".png") {
                    "image/png"
                } else {
                    "application/octet-stream"
                };
                tauri::http::Response::builder()
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Content-Type", mime)
                    .body(bytes)
                    .unwrap()
            } else {
                tauri::http::Response::builder()
                    .status(404)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Vec::new())
                    .unwrap()
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_data_dir,
            commands::import_model_folder,
            commands::capture_screen,
            commands::check_overlay_process,
            commands::kill_overlay_process,
            commands::get_process_memory,
            commands::open_models_folder,
            commands::notify_overlay_config_change,
            commands::start_overlay,
            commands::stop_overlay,
            commands::read_file_content,
            commands::read_file_base64,
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();

            let state = app.state::<AppState>();
            *state.main_window.lock().unwrap() = Some(main_window.clone());

            tray::setup_tray(app.handle())?;

            setup_global_shortcut(app.handle())?;

            let overlay_mgr = overlay_manager::OverlayManager::new(app.handle().clone());
            *state.overlay_manager.lock().unwrap() = Some(overlay_mgr);

            let app_handle = app.handle().clone();
            app.on_menu_event(move |_app, event| {
                let _ = tray::handle_menu_event(&app_handle, event.id().as_ref());
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings = window.state::<crate::commands::SettingsState>();
                if settings.0.lock().unwrap().close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyL);

    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let state = app.state::<AppState>();
                let mut mgr_guard = state.overlay_manager.lock().unwrap();
                if let Some(mgr) = mgr_guard.as_mut() {
                    let _ = mgr.toggle();
                }
            }
        })?;

    Ok(())
}

fn main() {
    install_panic_hook();
    setup_webview_folder();
    run();
}