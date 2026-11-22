# Live2D Mascot

Desktop Live2D mascot overlay + Master control app (OBS-style), built with **Tauri + Vue + PixiJS + pixi-live2d-display**.

## Structure

| Folder | Description |
|--------|-------------|
| `overlay/` | Transparent always-on-top overlay that renders the Live2D model on the desktop. |
| `master-app/` | Control panel to import models, configure position/scale/opacity, and start/stop the overlay. |

## Features

- 🖱️ **Hover-to-hide** — the mascot becomes invisible while the cursor is over it (toggle via `model.visible`).
- 🖱️ **Click expression** — click the mascot to trigger a random expression/motion.
- 🎛️ **Master control** — position, scale, opacity, FPS, always-on-top configured from the Master app.
- 🧊 **Smart click-through** — the overlay is click-through only outside the mascot's hit area.
- 📦 **Model import** — import Live2D model folders (`.model3.json` + textures + motions).

## Tech Stack

- **Tauri 2** (Rust) — native window + system integration
- **Vue 3** + Pinia — Master app UI
- **PixiJS 7** + **pixi-live2d-display 0.5** — Live2D rendering
- **Vite** — frontend build

## Getting Started

```bash
# Overlay
cd overlay
npm install
npm run build      # builds frontend + Tauri release exe

# Master app
cd ../master-app
npm install
npm run build
```

See `PROJECT_SUMMARY.md` for detailed architecture and API reference.