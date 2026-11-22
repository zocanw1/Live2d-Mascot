use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use crate::commands::OverlayConfig;

pub struct OverlayManager {
    app_handle: AppHandle,
    process: Mutex<Option<std::process::Child>>,
    config: Mutex<Option<OverlayConfig>>,
    start_time: Mutex<Option<u64>>,
    /// WebView2 user-data folder khusus instance overlay yang sedang berjalan.
    /// Dipisah per-launch supaya tidak bentrok (0x800700AA) dengan instance lain
    /// atau proses `msedgewebview2` yang masih tersisa.
    webview_dir: Mutex<Option<std::path::PathBuf>>,
}

impl OverlayManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            process: Mutex::new(None),
            config: Mutex::new(None),
            start_time: Mutex::new(None),
            webview_dir: Mutex::new(None),
        }
    }

    pub fn start(&self, config: OverlayConfig) -> Result<(), String> {
        // If already running, stop first
        if self.is_running() {
            let _ = self.stop();
        }

        *self.config.lock().unwrap() = Some(config.clone());

        // Get overlay executable path
        let overlay_exe = self.get_overlay_executable()?;

        // Build command arguments
        let args = vec![
            "--model".to_string(), config.model_path.clone(),
            "--config".to_string(), serde_json::to_string(&config).map_err(|e| e.to_string())?,
        ];

        // Siapkan folder WebView2 unik per-instance. Ini mencegah error
        // `WebView2 error 0x800700AA (The requested resource is in use)` yang
        // terjadi saat folder default dipakai bersama oleh instance overlay lain
        // atau proses `msedgewebview2` yang belum sepenuhnya keluar.
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let webview_dir = std::env::temp_dir()
            .join("live2d-mascot-overlay")
            .join(format!("webview-{}", nanos));
        let _ = std::fs::create_dir_all(&webview_dir);

        // Spawn overlay process
        let mut cmd = Command::new(&overlay_exe);
        cmd.args(&args)
            .env("WEBVIEW2_USER_DATA_FOLDER", &webview_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let mut child = cmd.spawn().map_err(|e| format!("Failed to start overlay: {}", e))?;
        let pid = child.id();

        // Health-check singkat: kalau overlay langsung mati (mis. gagal inisialisasi
        // WebView2), laporkan error yang jelas, bukan diam-diam gagal.
        std::thread::sleep(Duration::from_millis(500));
        if let Ok(Some(status)) = child.try_wait() {
            let _ = std::fs::remove_dir_all(&webview_dir);
            return Err(format!(
                "Overlay process exited immediately (status: {}). Pastikan executable overlay \
                 dan model Live2D valid, lalu coba lagi.",
                status
            ));
        }

        *self.webview_dir.lock().unwrap() = Some(webview_dir);
        *self.process.lock().unwrap() = Some(child);

        // Record start time for uptime tracking
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        *self.start_time.lock().unwrap() = Some(now);

        // Notify frontend
        self.app_handle.emit("overlay-started", serde_json::json!({ "pid": pid })).ok();

        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut process = self.process.lock().unwrap();
        
        if let Some(mut child) = process.take() {
            #[cfg(target_os = "windows")]
            {
                // /T memastikan seluruh process tree (termasuk msedgewebview2.exe)
                // ikut dimatikan, supaya folder WebView2 tidak terkunci.
                let _ = Command::new("taskkill")
                    .args(["/PID", &child.id().to_string(), "/T", "/F"])
                    .status();
            }
            
            #[cfg(not(target_os = "windows"))]
            {
                let _ = child.kill();
            }
            
            let _ = child.wait();
        }
        
        self.app_handle.emit("overlay-stopped", ()).ok();

        // Clear start time
        *self.start_time.lock().unwrap() = None;

        // Hapus folder WebView2 milik instance yang baru berhenti (best-effort,
        // karena proses msedgewebview2 bisa sesaat masih melepas handle folder).
        let webview_dir = self.webview_dir.lock().unwrap().take();
        if let Some(dir) = webview_dir {
            for _ in 0..5 {
                if std::fs::remove_dir_all(&dir).is_ok() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }

        Ok(())
    }

    pub fn get_start_time(&self) -> Option<u64> {
        self.start_time.lock().unwrap().clone()
    }

    pub fn toggle(&self) -> Result<(), String> {
        if self.is_running() {
            self.stop()
        } else {
            if let Some(config) = self.config.lock().unwrap().clone() {
                self.start(config)
            } else {
                Err("No configuration available".into())
            }
        }
    }

    pub fn is_running(&self) -> bool {
        let mut process = self.process.lock().unwrap();
        if let Some(ref mut child) = *process {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *process = None;
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn get_pid(&self) -> Option<u32> {
        self.process.lock().unwrap().as_ref().map(|p| p.id())
    }

    fn get_overlay_executable(&self) -> Result<String, String> {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_dir = current_exe.parent().ok_or("Failed to get exe directory")?;
        
        #[cfg(target_os = "windows")]
        let overlay_name = "live2d-mascot-overlay.exe";
        #[cfg(not(target_os = "windows"))]
        let overlay_name = "live2d-mascot-overlay";
        
        let candidates = vec![
            exe_dir.join(overlay_name),
            exe_dir.join("..").join("..").join("overlay").join("src-tauri").join("target").join("release").join(overlay_name),
            exe_dir.join("..").join("..").join("..").join("overlay").join("src-tauri").join("target").join("release").join(overlay_name),
            exe_dir.join("..").join("..").join("overlay").join("target").join("release").join(overlay_name),
            std::path::PathBuf::from(r"C:\coding\AI\Star-Stella\live2d-mascot\overlay\src-tauri\target\release").join(overlay_name),
        ];

        for path in candidates {
            if path.exists() {
                return Ok(path.to_string_lossy().to_string());
            }
        }

        Err(format!("Overlay executable {} not found in candidate paths", overlay_name))
    }
}
