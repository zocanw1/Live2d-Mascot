use std::sync::Mutex;
use std::io::Write;

fn log_trace(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(r"C:\coding\AI\Star-Stella\live2d-mascot\trace.log") {
        let _ = writeln!(f, "{}", msg);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct OverlayConfig {
    #[serde(rename = "modelId", default)]
    pub model_id: String,
    #[serde(rename = "modelPath", default)]
    pub model_path: String,
    #[serde(default = "default_pos")]
    pub position: Position,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(rename = "lookAtEnabled", default = "default_true")]
    pub look_at_enabled: bool,
    #[serde(rename = "clickThroughLock", default)]
    pub click_through_lock: bool,
    #[serde(rename = "fpsLimit", default = "default_fps")]
    pub fps_limit: u32,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(rename = "alwaysOnTop", default = "default_true")]
    pub always_on_top: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

fn default_pos() -> Position {
    Position { x: 960.0, y: 540.0 }
}
fn default_scale() -> f64 { 0.2 }
fn default_fps() -> u32 { 60 }
fn default_opacity() -> f64 { 1.0 }
fn default_true() -> bool { true }

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InitData {
    pub model_path: String,
    pub config: OverlayConfig,
}

#[tauri::command]
fn log_js(msg: String) {
    log_trace(&format!("[JS] {}", msg));
}

#[tauri::command]
fn set_ignore_cursor_events(window: tauri::WebviewWindow, ignore: bool) -> Result<(), String> {
    log_trace(&format!("set_ignore_cursor_events: {}", ignore));
    window.set_ignore_cursor_events(ignore).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_init_data(state: tauri::State<Mutex<InitData>>) -> Result<InitData, String> {
    log_trace("get_init_data called");
    Ok(state.lock().unwrap().clone())
}

/// Posisi kursor OS (pixel fisik layar). Tetap terbaca walau window overlay
/// sedang click-through, sehingga look-at bisa mengikuti mouse ke seluruh layar.
#[tauri::command]
fn get_cursor_pos() -> (i32, i32) {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        let mut p = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut p);
        (p.x, p.y)
    }
    #[cfg(not(target_os = "windows"))]
    {
        (0, 0)
    }
}

pub fn run() {
    log_trace("run() called");

    let args: Vec<String> = std::env::args().collect();
    log_trace(&format!("args: {:?}", args));

    let mut model_path = String::new();
    let mut config_str = String::new();

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--model" && i + 1 < args.len() {
            model_path = args[i + 1].clone();
            i += 1;
        } else if args[i] == "--config" && i + 1 < args.len() {
            config_str = args[i + 1].clone();
            i += 1;
        }
        i += 1;
    }

    let default_model = r#"C:\Users\ADVAN\Downloads\IceGirl_Live2d\IceGIrl Live2D\IceGirl.model3.json"#.to_string();
    if model_path.is_empty() {
        model_path = default_model;
    }

    let config: OverlayConfig = if !config_str.is_empty() {
        serde_json::from_str(&config_str).unwrap_or_else(|_| OverlayConfig {
            model_id: "IceGirl".to_string(),
            model_path: model_path.clone(),
            position: default_pos(),
            scale: 0.2,
            look_at_enabled: true,
            click_through_lock: false,
            fps_limit: 60,
            opacity: 1.0,
            always_on_top: true,
        })
    } else {
        OverlayConfig {
            model_id: "IceGirl".to_string(),
            model_path: model_path.clone(),
            position: default_pos(),
            scale: 0.2,
            look_at_enabled: true,
            click_through_lock: false,
            fps_limit: 60,
            opacity: 1.0,
            always_on_top: true,
        }
    };

    let init_data = InitData {
        model_path: model_path.clone(),
        config,
    };

    log_trace("building tauri app");
    let result = tauri::Builder::default()
        .register_uri_scheme_protocol("live2d", |_app, request| {
            let path = request.uri().path();
            log_trace(&format!("live2d protocol request: {}", path));
            let decoded = percent_encoding::percent_decode_str(path).decode_utf8_lossy().to_string();
            let trimmed = decoded.trim_start_matches('/');
            let file_path = std::path::Path::new(trimmed);

            if let Ok(bytes) = std::fs::read(file_path) {
                let mime = if trimmed.ends_with(".json") {
                    "application/json"
                } else if trimmed.ends_with(".png") {
                    "image/png"
                } else if trimmed.ends_with(".moc3") {
                    "application/octet-stream"
                } else {
                    "application/octet-stream"
                };

                tauri::http::Response::builder()
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Content-Type", mime)
                    .body(bytes)
                    .unwrap()
            } else {
                log_trace(&format!("404 for path: {:?}", file_path));
                tauri::http::Response::builder()
                    .status(404)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Vec::new())
                    .unwrap()
            }
        })
        .manage(Mutex::new(init_data))
        .invoke_handler(tauri::generate_handler![
            log_js,
            set_ignore_cursor_events,
            get_init_data,
            get_cursor_pos,
        ])
        .setup(|_app| {
            log_trace("setup hook called");
            Ok(())
        })
        .on_window_event(|_window, event| {
            log_trace(&format!("window event: {:?}", event));
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        let msg = format!("RUN ERROR in overlay: {}", e);
        log_trace(&msg);
        let _ = std::fs::write(r"C:\coding\AI\Star-Stella\live2d-mascot\error.log", msg);
    } else {
        log_trace("run() completed cleanly");
    }
}

fn main() {
    let _ = std::fs::remove_file(r"C:\coding\AI\Star-Stella\live2d-mascot\trace.log");
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC in overlay: {}", info);
        log_trace(&msg);
        let _ = std::fs::write(r"C:\coding\AI\Star-Stella\live2d-mascot\panic.log", msg);
    }));

    log_trace("main() started");
    run();
}
