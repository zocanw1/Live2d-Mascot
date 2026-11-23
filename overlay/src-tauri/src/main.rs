use std::sync::Mutex;
use std::io::Write;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// State untuk menyimpan shortcut PTT yang sedang aktif (agar bisa di-unregister
/// saat hotkey diganti).
struct PttState {
    current: Mutex<Option<Shortcut>>,
}

// ===== Push-To-Talk via tombol mouse samping (Mouse4/Mouse5) =====
// tauri-plugin-global-shortcut hanya mendukung tombol KEYBOARD, sehingga tombol
// mouse samping (XBUTTON1/XBUTTON2) tidak bisa dibind lewat hotkey biasa. Kita
// pasang low-level mouse hook (WH_MOUSE_LL) yang mendeteksi tombol samping secara
// global dan meng-emit "ptt-down"/"ptt-up" ke window overlay.
//
// MOUSE_PTT: 0 = nonaktif, 1 = Mouse4 (XBUTTON1/Back), 2 = Mouse5 (XBUTTON2/Forward).
// Diubah via command set_mouse_ptt dari JS (tanpa perlu restart hook).
#[cfg(target_os = "windows")]
static MOUSE_PTT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
#[cfg(target_os = "windows")]
static MOUSE_APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

#[cfg(target_os = "windows")]
unsafe extern "system" fn mouse_ptt_proc(
    ncode: i32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, MSLLHOOKSTRUCT, HHOOK, WM_XBUTTONDOWN, WM_XBUTTONUP,
    };
    // ncode == HC_ACTION (0) -> ada event mouse.
    if ncode == 0 {
        let id = wparam.0 as u32;
        if id == WM_XBUTTONDOWN || id == WM_XBUTTONUP {
            // mouseData high-word: 1 = XBUTTON1 (Mouse4), 2 = XBUTTON2 (Mouse5).
            let data = (*(lparam.0 as *const MSLLHOOKSTRUCT)).mouseData;
            let which = ((data >> 16) & 0xffff) as u32;
            let cfg = MOUSE_PTT.load(std::sync::atomic::Ordering::Relaxed);
            if cfg != 0 && which == cfg {
                if let Some(app) = MOUSE_APP.get() {
                    if let Some(win) = app.get_webview_window("main") {
                        if id == WM_XBUTTONDOWN {
                            let _ = win.emit("ptt-down", ());
                        } else {
                            let _ = win.emit("ptt-up", ());
                        }
                    }
                }
            }
        }
    }
    CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
}

/// Pasang WH_MOUSE_LL hook pada thread khusus (butuh message loop sendiri).
#[cfg(target_os = "windows")]
fn start_mouse_ptt_hook(app: AppHandle) {
    use windows::Win32::Foundation::{HINSTANCE, HWND};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, MSG, WH_MOUSE_LL,
    };
    let _ = MOUSE_APP.set(app);
    std::thread::spawn(move || unsafe {
        let hmod = GetModuleHandleW(None).unwrap_or_default();
        // Simpan handle hook selama proses hidup (tidak di-unhook).
        let _hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_ptt_proc), HINSTANCE(hmod.0), 0);
        log_trace("mouse PTT hook installed");
        let mut msg = MSG::default();
        loop {
            let r = GetMessageW(&mut msg, HWND::default(), 0, 0);
            if r.0 <= 0 {
                break; // 0 = WM_QUIT, -1 = error
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        log_trace("mouse PTT message loop ended");
    });
}

/// Atur tombol mouse samping untuk push-to-talk.
/// `button`: "" (nonaktif) | "Mouse4"/"Back" (XBUTTON1) | "Mouse5"/"Forward" (XBUTTON2).
#[tauri::command]
fn set_mouse_ptt(button: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let code = match button.trim().to_ascii_lowercase().as_str() {
            "" | "none" => 0u32,
            "mouse4" | "xbutton1" | "back" => 1,
            "mouse5" | "xbutton2" | "forward" => 2,
            _ => 0,
        };
        MOUSE_PTT.store(code, std::sync::atomic::Ordering::SeqCst);
        log_trace(&format!("set_mouse_ptt: {:?} -> {}", button, code));
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = button;
    }
    Ok(())
}

/// Konversi nama tombol (mis. "Space", "E", "F5") ke Code.
fn key_to_code(key: &str) -> Option<Code> {
    let bytes = key.as_bytes();
    if bytes.len() == 1 {
        let c = bytes[0].to_ascii_uppercase();
        let code = match c {
            b'A' => Code::KeyA, b'B' => Code::KeyB, b'C' => Code::KeyC, b'D' => Code::KeyD,
            b'E' => Code::KeyE, b'F' => Code::KeyF, b'G' => Code::KeyG, b'H' => Code::KeyH,
            b'I' => Code::KeyI, b'J' => Code::KeyJ, b'K' => Code::KeyK, b'L' => Code::KeyL,
            b'M' => Code::KeyM, b'N' => Code::KeyN, b'O' => Code::KeyO, b'P' => Code::KeyP,
            b'Q' => Code::KeyQ, b'R' => Code::KeyR, b'S' => Code::KeyS, b'T' => Code::KeyT,
            b'U' => Code::KeyU, b'V' => Code::KeyV, b'W' => Code::KeyW, b'X' => Code::KeyX,
            b'Y' => Code::KeyY, b'Z' => Code::KeyZ,
            b'0' => Code::Digit0, b'1' => Code::Digit1, b'2' => Code::Digit2, b'3' => Code::Digit3,
            b'4' => Code::Digit4, b'5' => Code::Digit5, b'6' => Code::Digit6, b'7' => Code::Digit7,
            b'8' => Code::Digit8, b'9' => Code::Digit9,
            _ => return None,
        };
        return Some(code);
    }
    match key {
        "Space" => Some(Code::Space),
        "Enter" => Some(Code::Enter),
        "Escape" => Some(Code::Escape),
        "Tab" => Some(Code::Tab),
        "Backspace" => Some(Code::Backspace),
        "ArrowUp" => Some(Code::ArrowUp),
        "ArrowDown" => Some(Code::ArrowDown),
        "ArrowLeft" => Some(Code::ArrowLeft),
        "ArrowRight" => Some(Code::ArrowRight),
        "F1" => Some(Code::F1), "F2" => Some(Code::F2), "F3" => Some(Code::F3), "F4" => Some(Code::F4),
        "F5" => Some(Code::F5), "F6" => Some(Code::F6), "F7" => Some(Code::F7), "F8" => Some(Code::F8),
        "F9" => Some(Code::F9), "F10" => Some(Code::F10), "F11" => Some(Code::F11), "F12" => Some(Code::F12),
        _ => None,
    }
}

/// Parse string hotkey (mis. "Ctrl+Shift+Space") menjadi Shortcut.
fn parse_hotkey(s: &str) -> Option<Shortcut> {
    let mut modifiers = Modifiers::empty();
    let mut code: Option<Code> = None;
    for part in s.split('+') {
        match part.trim() {
            "Ctrl" | "Control" => modifiers |= Modifiers::CONTROL,
            "Shift" => modifiers |= Modifiers::SHIFT,
            "Alt" => modifiers |= Modifiers::ALT,
            "Super" | "Win" | "Meta" => modifiers |= Modifiers::SUPER,
            key => {
                code = key_to_code(key);
            }
        }
    }
    code.map(|c| Shortcut::new(Some(modifiers), c))
}

/// Daftarkan/ ganti shortcut push-to-talk secara dinamis dari hotkey string.
#[tauri::command]
fn set_ptt_shortcut(app: AppHandle, state: State<'_, PttState>, hotkey: String) -> Result<(), String> {
    // Unregister shortcut lama (bila ada).
    if let Some(old) = state.current.lock().unwrap().take() {
        let _ = app.global_shortcut().unregister(old);
    }
    // Hotkey kosong = nonaktifkan shortcut keyboard (mis. saat PTT pakai tombol mouse).
    if hotkey.trim().is_empty() {
        log_trace("ptt shortcut cleared");
        return Ok(());
    }
    let shortcut = parse_hotkey(&hotkey)
        .ok_or_else(|| format!("invalid hotkey: {}", hotkey))?;
    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if let Some(win) = app_handle.get_webview_window("main") {
                match event.state {
                    ShortcutState::Pressed => {
                        let _ = win.emit("ptt-down", ());
                    }
                    ShortcutState::Released => {
                        let _ = win.emit("ptt-up", ());
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;
    *state.current.lock().unwrap() = Some(shortcut);
    log_trace(&format!("ptt shortcut set: {}", hotkey));
    Ok(())
}

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

/// Baca runtime config yang ditulis Master (untuk live-update lookAtEnabled
/// dll). Mengembalikan string kosong bila file belum ada.
#[tauri::command]
fn get_runtime_config() -> String {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let path = std::path::Path::new(&appdata)
        .join("live2d-mascot")
        .join("overlay-runtime.json");
    match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => String::new(),
    }
}

/// Parse nilai JSON pertama dari sebuah string, mengabaikan karakter/JSON lain
/// di belakangnya. Diperlukan karena sebagian gateway LLM (mis. localhost:20128)
/// mengembalikan objek JSON diikuti penanda SSE `data: [DONE]` -> kalau di-parse
/// strict akan error "trailing characters".
fn parse_first_json(s: &str) -> Option<serde_json::Value> {
    serde_json::Deserializer::from_str(s)
        .into_iter::<serde_json::Value>()
        .next()
        .and_then(Result::ok)
}

/// Transkripsi audio ke teks via Groq Whisper API.
/// `audio` = byte audio (webm/ogg/wav/mp3), `api_key` = Groq key, `model` = whisper model.
const GROQ_TRANSCRIBE_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

#[tauri::command]
async fn transcribe_audio(audio: Vec<u8>, api_key: String, model: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let part = reqwest::multipart::Part::bytes(audio)
        .file_name("audio.webm")
        .mime_str("audio/webm")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new()
        .text("model", model)
        .text("response_format", "json".to_string())
        .part("file", part);

    let resp = client
        .post(GROQ_TRANSCRIBE_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("transcribe request failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("groq error {}: {}", status, text));
    }
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(json["text"].as_str().unwrap_or("").to_string())
}

/// Kirim chat ke LLM (OpenAI-compatible endpoint) dan kembalikan balasan teks.
/// `messages_json` = JSON array messages, `endpoint` = base URL (mis. http://localhost:20128/v1).
#[tauri::command]
async fn chat_completion(
    messages_json: String,
    endpoint: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let messages: serde_json::Value =
        serde_json::from_str(&messages_json).map_err(|e| e.to_string())?;
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
    });

    let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("llm request failed: {}", e))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("llm error {}: {}", status, text));
    }
    // Toleran terhadap trailing data (mis. `data: [DONE]` dari gateway lokal).
    let json: serde_json::Value = parse_first_json(&text)
        .ok_or_else(|| "llm response bukan JSON valid".to_string())?;
    Ok(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string())
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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
        .manage(PttState { current: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            log_js,
            set_ignore_cursor_events,
            get_init_data,
            get_cursor_pos,
            get_runtime_config,
            transcribe_audio,
            chat_completion,
            set_ptt_shortcut,
            set_mouse_ptt,
        ])
        .setup(|app| {
            log_trace("setup hook called");

            // Pasang low-level mouse hook untuk PTT tombol mouse samping.
            #[cfg(target_os = "windows")]
            start_mouse_ptt_hook(app.handle().clone());

            // Buang title bar putih & buat window menutupi seluruh monitor.
            // (fullscreen:true di config menyebabkan title bar muncul di Windows
            // untuk window transparan; kita pakai borderless + size monitor).
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_decorations(false);
                if let Ok(Some(monitor)) = win.primary_monitor() {
                    let size = monitor.size();
                    let pos = monitor.position();
                    let _ = win.set_size(tauri::PhysicalSize::<u32>::new(size.width, size.height));
                    let _ = win.set_position(tauri::PhysicalPosition::<i32>::new(pos.x, pos.y));
                    log_trace(&format!("window sized to monitor {}x{}", size.width, size.height));
                }
            }

            // Global hotkey Ctrl+Shift+E -> picu ekspresi acak.
            // Window overlay selalu click-through (klik menembus ke app di
            // belakang), jadi ekspresi dipicu via hotkey, bukan klik mouse.
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyE);
            let app_handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.emit("trigger-expression", ());
                            log_trace("hotkey trigger-expression emitted");
                        }
                    }
                })?;

            // Push-to-talk hotkey didaftarkan dinamis via command set_ptt_shortcut
            // (dipanggil dari JS saat voiceConfig.pttHotkey berubah).
            log_trace("global shortcuts registered");
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
