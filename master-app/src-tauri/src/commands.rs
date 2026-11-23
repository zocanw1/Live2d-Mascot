use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, System};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OverlayConfig {
    #[serde(rename = "modelId", default)]
    pub model_id: String,
    #[serde(rename = "modelPath", default)]
    pub model_path: String,
    #[serde(default = "default_position")]
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

fn default_position() -> Position { Position { x: 400.0, y: 300.0 } }
fn default_scale() -> f64 { 0.25 }
fn default_true() -> bool { true }
fn default_fps() -> u32 { 60 }
fn default_opacity() -> f64 { 1.0 }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub struct SettingsState(pub Mutex<AppSettings>);

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub auto_start_overlay: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub overlay_config_path: String,
    pub models_directory: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OverlayStatus {
    pub is_running: bool,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f32>,
}

#[tauri::command]
pub fn get_app_data_dir() -> Result<String, String> {
    let dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("live2d-mascot");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

/// Simpan file config (mis. settings.json / overlay-config.json) ke folder
/// data app. Dipakai frontend agar save/load tidak bergantung pada path-scope
/// plugin fs (yang sempat gagal karena path dobel-nested).
#[tauri::command]
pub fn save_config_file(filename: String, content: String) -> Result<(), String> {
    let dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("live2d-mascot");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&filename);
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

/// Baca file config dari folder data app. Bila file belum ada, kembalikan
/// string kosong supaya caller memakai default.
#[tauri::command]
pub fn load_config_file(filename: String) -> Result<String, String> {
    let dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("live2d-mascot");
    let path = dir.join(&filename);
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(_) => Ok(String::new()),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenCapture {
    /// Data URL PNG (data:image/png;base64,...) dari screenshot monitor utama.
    pub data_url: String,
    pub width: u32,
    pub height: u32,
}

/// Screenshot monitor utama. Main window master disembunyikan sesaat sebelum
/// capture supaya hasilnya bersih (tanpa panel master), lalu ditampilkan lagi.
#[tauri::command]
pub async fn capture_screen(app: AppHandle) -> Result<ScreenCapture, String> {
    use tauri::Manager;

    let win = app.get_webview_window("main");
    if let Some(w) = &win {
        let _ = w.hide();
    }
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    let result = (|| -> Result<ScreenCapture, String> {
        use xcap::Monitor;
        let monitors = Monitor::all().map_err(|e| e.to_string())?;
        let monitor = monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| monitors.first())
            .ok_or("Tidak ada monitor ditemukan")?;

        let image = monitor.capture_image().map_err(|e| e.to_string())?;
        let width = image.width();
        let height = image.height();

        let tmp = std::env::temp_dir().join(format!(
            "live2d-screenshot-{}.png",
            std::process::id()
        ));
        image.save(&tmp).map_err(|e| e.to_string())?;
        let bytes = std::fs::read(&tmp).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&tmp);

        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(ScreenCapture {
            data_url: format!("data:image/png;base64,{}", b64),
            width,
            height,
        })
    })();

    if let Some(w) = &win {
        let _ = w.show();
        let _ = w.set_focus();
    }

    result
}

/// Cari file `.model3.json` di dalam sebuah folder (root dulu, lalu subfolder).
fn find_model3_in_dir(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    // Prioritaskan file di root folder.
    for e in entries.flatten() {
        let p = e.path();
        if p.is_file() {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".model3.json") || name == "model3.json" {
                    return Some(p);
                }
            }
        }
    }
    // Lalu cari di subfolder (rekursif, kedalaman terbatas).
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() {
            if let Some(found) = find_model3_in_dir(&p) {
                return Some(found);
            }
        }
    }
    None
}

/// Salin isi folder `src` ke `dest` secara rekursif.
fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Import model Live2D berbasis folder.
///
/// Menyalin SELURUH folder model (model3.json, moc3, textures, physics, motions,
/// expressions, dst.) ke `AppData/live2d-mascot/models/<nama-folder>/`, lalu
/// mengembalikan path absolut ke `.model3.json` di lokasi baru. Dengan begitu
/// model tetap ada setelah app di-restart dan tidak bergantung pada lokasi asal.
#[tauri::command]
pub async fn import_model_folder(source_dir: String) -> Result<String, String> {
    let src = std::path::PathBuf::from(&source_dir);
    if !src.is_dir() {
        return Err(format!("Path bukan sebuah folder: {}", source_dir));
    }

    let model_file = find_model3_in_dir(&src)
        .ok_or_else(|| "Folder tidak berisi file .model3.json".to_string())?;

    let base = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("live2d-mascot")
        .join("models");
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;

    let folder_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("model");

    // Hindari menimpa: kalau folder tujuan sudah ada, tambahkan suffix angka.
    let mut dest_dir = base.join(folder_name);
    if dest_dir.exists() {
        let mut n = 1;
        loop {
            dest_dir = base.join(format!("{}-{}", folder_name, n));
            if !dest_dir.exists() {
                break;
            }
            n += 1;
        }
    }

    copy_dir_recursive(&src, &dest_dir)?;

    // Path model3.json relatif terhadap folder sumber, lalu dipetakan ke tujuan.
    let rel = model_file
        .strip_prefix(&src)
        .map_err(|e| e.to_string())?;
    let dest_model = dest_dir.join(rel);

    Ok(dest_model.to_string_lossy().to_string())
}

#[tauri::command]
pub fn check_overlay_process(state: State<'_, AppState>) -> Result<OverlayStatus, String> {
    let mgr_guard = state.overlay_manager.lock().unwrap();
    if let Some(mgr) = mgr_guard.as_ref() {
        let is_running = mgr.is_running();
        let pid = mgr.get_pid();

        let mut status = OverlayStatus {
            is_running,
            pid,
            uptime: None,
            memory_usage: None,
            cpu_usage: None,
        };

        if let Some(pid) = pid {
            if let Ok(mem) = get_process_memory_inner(pid) {
                status.memory_usage = Some(mem);
            }
            if let Ok(cpu) = get_process_cpu_inner(pid) {
                status.cpu_usage = Some(cpu);
            }
            // Uptime: approximate since process start (sysinfo doesn't expose start_time directly on all platforms,
            // so we track it via overlay manager start time)
            if let Some(start_time) = mgr.get_start_time() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if now >= start_time {
                    status.uptime = Some(now - start_time);
                }
            }
        }

        Ok(status)
    } else {
        Ok(OverlayStatus::default())
    }
}

/// Mendapatkan memory usage (bytes) untuk process dengan PID tertentu.
fn get_process_memory_inner(pid: u32) -> Result<u64, String> {
    let mut sys = System::new();
    let refresh_kind = ProcessRefreshKind::new().with_memory();
    sys.refresh_processes_specifics(refresh_kind);
    let pid_sys = sysinfo::Pid::from_u32(pid);
    if let Some(proc) = sys.process(pid_sys) {
        Ok(proc.memory())
    } else {
        Err(format!("Process with PID {} not found", pid))
    }
}

/// Mendapatkan CPU usage (persen) untuk process dengan PID tertentu.
fn get_process_cpu_inner(pid: u32) -> Result<f32, String> {
    let mut sys = System::new();
    let refresh_kind = ProcessRefreshKind::new().with_cpu();
    sys.refresh_processes_specifics(refresh_kind);
    // sysinfo perlu dua kali refresh untuk mendapatkan akurasi CPU, tapi untuk estimasi awal cukup satu kali
    let pid_sys = sysinfo::Pid::from_u32(pid);
    if let Some(proc) = sys.process(pid_sys) {
        Ok(proc.cpu_usage())
    } else {
        Err(format!("Process with PID {} not found", pid))
    }
}

#[tauri::command]
pub fn get_process_memory(pid: u32) -> Result<u64, String> {
    get_process_memory_inner(pid)
}

#[tauri::command]
pub async fn start_overlay(state: State<'_, AppState>, model_path: String, config: OverlayConfig) -> Result<(), String> {
    let mut mgr_guard = state.overlay_manager.lock().unwrap();
    if let Some(mgr) = mgr_guard.as_mut() {
        let mut cfg = config;
        cfg.model_path = model_path;
        mgr.start(cfg)
    } else {
        Err("Overlay manager not initialized".into())
    }
}

#[tauri::command]
pub fn stop_overlay(state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr_guard = state.overlay_manager.lock().unwrap();
    if let Some(mgr) = mgr_guard.as_mut() {
        mgr.stop()
    } else {
        Err("Overlay manager not initialized".into())
    }
}

#[tauri::command]
pub fn kill_overlay_process(state: State<'_, AppState>) -> Result<(), String> {
    stop_overlay(state)
}

#[tauri::command]
pub fn open_models_folder(_app: AppHandle) -> Result<(), String> {
    let dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("live2d-mascot")
        .join("models");
    
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn notify_overlay_config_change(config: serde_json::Value) -> Result<(), String> {
    // Tulis runtime config (raw JSON) agar overlay yang sedang berjalan bisa
    // membaca perubahan (lookAtEnabled, opacity, voice, dll) secara live.
    let dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("live2d-mascot");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("overlay-runtime.json");
    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    println!("Overlay config changed");
    Ok(())
}

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_file_base64(path: String) -> Result<String, String> {
    use base64::Engine;
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let mime = if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "application/octet-stream"
    };
    Ok(format!("data:{};base64,{}", mime, encoded))
}
