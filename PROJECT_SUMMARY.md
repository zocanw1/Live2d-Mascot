# 📋 Ringkasan Project: Live2D Desktop Mascot

**Nama Project:** Live2D Desktop Mascot  
**Lokasi:** `C:\coding\AI\Star-Stella\live2d-mascot\`  
**Status:** Build selesai (Overlay) / Master App build diperlukan (dependencies baru: sysinfo)  
**Bahasa UI:** Indonesia

---

## 🎯 Deskripsi Singkat

Aplikasi desktop standalone untuk menampilkan karakter Live2D (`.model3.json`) sebagai mascot di layar Windows. Terdiri dari **2 aplikasi terpisah**:

1. **Master App** — Panel kontrol seperti OBS Studio (atur model, posisi, FPS, tombol Start/Stop, sistem tray).
2. **Overlay App** — Renderer ringan yang menampilkan karakter Live2D di atas desktop (transparan, always-on-top, click-through).

---

## 📁 Struktur Folder

```
live2d-mascot/
├── master-app/              # Panel Kontrol (Master)
│   ├── src/                 # Vue 3 + TypeScript frontend
│   ├── src-tauri/           # Rust backend (Tauri v2)
│   ├── dist/                # Hasil build frontend
│   └── target/release/      # Executable master (jika build berhasil)
│       └── live2d-mascot-master.exe
├── overlay/                 # Overlay Renderer
│   ├── src/                 # PixiJS + pixi-live2d-display
│   ├── src-tauri/           # Minimal Rust (Tauri v2)
│   └── target/release/      # Executable overlay (SUDAH BUILD)
│       ├── live2d-mascot-overlay.exe
│       └── bundle/
│           ├── msi/
│           └── nsis/
└── PROJECT_SUMMARY.md       # File ini
```

---

## ✅ Fitur Master App (Panel Kontrol)

| Fitur | Keterangan |
|-------|------------|
| Dashboard | Statistik, quick actions, preview model aktif |
| Manajemen Model | Import, cari, hapus, preview model Live2D |
| Pengaturan | Posisi, skala, FPS, look-at, click-through lock, opacity, tema, bahasa |
| Kontrol Overlay | Start / Stop / Restart dengan status live |
| System Tray | Minimize ke tray, show/hide, quit |
| Persistensi Config | Disimpan di `%APPDATA%/live2d-mascot/config.json` |
| Auto-discovery Model | Scan otomatis folder models untuk `.model3.json` |

---

## ✅ Fitur Overlay App (Renderer)

| Fitur | Keterangan |
|-------|------------|
| Window transparan, tanpa bingkai | Hanya karakter yang terlihat |
| Always on top (`HWND_TOPMOST`) | Selalu di depan semua aplikasi |
| Stealth mode (`WS_EX_TOOLWINDOW`) | Tersembunyi dari taskbar & Alt+Tab |
| Smart click-through | Klik tembus ke aplikasi belakang saat area kosong |
| Drag geser + Ctrl+Scroll zoom | Pindah posisi & ubah ukuran karakter |
| Global lock (`Ctrl+Shift+L`) | Kunci klik/geser saat fokus kerja/game |
| Mouse look-at | Mata mengikuti kursor mouse |
| Tap reactions | Ekspresi/motion acak saat diklik |
| Physics, auto-blink, idle motions | Animasi otomatis tanpa input pengguna |

---

## 🎮 Model Yang Di-load

**Path model:**  
`C:\Users\ADVAN\Downloads\IceGirl_Live2d\IceGIrl Live2D\IceGirl.model3.json`

---

## 🔧 Tech Stack

| Layer | Teknologi |
|-------|-----------|
| Frontend Master | Vue 3 + TypeScript + Pinia + Vue Router |
| Backend Master | Rust + Tauri v2 |
| Renderer Overlay | PixiJS 7 + pixi-live2d-display + live2dcubismcore |
| IPC / Komunikasi | Tauri commands + file config JSON + CLI args |

---

## 🚀 Cara Menjalankan

### Langkah 1: Jalankan Master App
```
C:\coding\AI\Star-Stella\live2d-mascot\master-app\target\release\live2d-mascot-master.exe
```

### Langkah 2: Import Model
Klik "Import Model" → pilih `IceGirl.model3.json`

### Langkah 3: Start Overlay
Klik tombol **"Start Overlay"** → karakter muncul di layar

### Kontrol Cepat
- `Ctrl + Scroll` = zoom in/out
- Drag karakter = geser posisi
- `Ctrl + Shift + L` = toggle click-through lock
- Klik kanan tray icon = kontrol cepat

---

## ⚠️ Catatan Penting

- **Overlay app** (`live2d-mascot-overlay.exe`) **SUDAH BUILD** dan siap pakai di `overlay\target\release\`.
- **Master app** (`live2d-mascot-master.exe`) build-nya **GAGAL** saat ini karena error fitur `dialog` di `Cargo.toml`. Perlu diperbaiki dulu sebelum file `.exe`-nya tersedia.
- File model harus tetap di lokasi aslinya agar dapat di-load oleh master app.

---

## 📦 Installer (Bundle)

Jika build berhasil, installer ada di:
```
master-app\target\release\bundle\msi\Live2D Mascot Master_1.0.0_x64_en-US.msi
master-app\target\release\bundle\nsis\Live2D Mascot Master_1.0.0_x64-setup.exe
```

Dan untuk overlay:
```
overlay\target\release\bundle\msi\Live2D Mascot Overlay_1.0.0_x64_en-US.msi
overlay\target\release\bundle\nsis\Live2D Mascot Overlay_1.0.0_x64-setup.exe
```

---

Dibuat oleh Stella Agent untuk Zocan (Zocanw/)  
Project course assignment untuk Pak Halim


---
## 🔧 Perbaruan Terbaru (Memory/Process Monitoring)

- **`get_process_memory`** diimplementasikan sepenuhnya menggunakan crate `sysinfo` (mengambil memory usage dalam bytes berdasarkan PID).
- **`check_overlay_process`** sekarang mengisi `memory_usage`, `cpu_usage`, dan `uptime` (dibandingkan sebelumnya yang selalu `None` / `0`).
- **`OverlayManager`** sekarang mencatat `start_time` saat overlay dimulai dan menghapusnya saat dihentikan, digunakan untuk perhitungan uptime.
- Dependency `sysinfo = "0.30"` ditambahkan ke `master-app/src-tauri/Cargo.toml`.

## ⚠️ Masalah Masih Terbuka

- **WebView2 crash (0x800700AA):** Overlay app crash saat startup dengan error "The requested resource is in use". Kemungkinan disebabkan oleh WebView2 Runtime belum terinstal atau corrupted. Solusi: install ulang Microsoft Edge WebView2 Runtime dari https://aka.ms/WebView2Bootstrap
- **Master app build:** Perlu di-rebuild setelah menambahkan dependency `sysinfo`. Jalankan `cargo build` di `master-app/src-tauri/`.