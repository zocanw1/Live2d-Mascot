import { Application, Ticker } from 'pixi.js';
import { Live2DModel, config as live2dConfig } from 'pixi-live2d-display';
import { invoke } from '@tauri-apps/api/core';

// Bridge JS logs to Rust trace logger
const origLog = console.log;
const origErr = console.error;
const origWarn = console.warn;
function report(type: string, ...args: any[]) {
  const msg = `[${type}] ` + args.map(a => {
    if (a instanceof Error) return a.stack || a.message;
    if (typeof a === 'object') {
      try { return JSON.stringify(a); } catch { return String(a); }
    }
    return String(a);
  }).join(' ');
  invoke('log_js', { msg }).catch(() => {});
}
console.log = (...args) => { origLog(...args); report('LOG', ...args); };
console.error = (...args) => { origErr(...args); report('ERR', ...args); };
console.warn = (...args) => { origWarn(...args); report('WARN', ...args); };
window.addEventListener('error', (e) => report('WINDOW_ERR', e.message, e.filename, e.lineno, e.error?.stack));
window.addEventListener('unhandledrejection', (e) => report('UNHANDLED_REJECTION', e.reason?.stack || e.reason));

// Polyfill Live2DCubismCore.Memory for pixi-live2d-display compatibility
if (typeof window !== 'undefined') {
  const Core = (window as any).Live2DCubismCore;
  if (Core && !Core.Memory) {
    Core.Memory = {
      initializeAmountOfMemory: () => {},
    };
  }
}

// Register Ticker with Live2DModel
Live2DModel.registerTicker(Ticker);

interface OverlayConfig {
  modelId: string;
  modelPath: string;
  position: { x: number; y: number };
  scale: number;
  lookAtEnabled: boolean;
  clickThroughLock: boolean;
  fpsLimit: 30 | 60;
  opacity: number;
  alwaysOnTop: boolean;
}

let config: OverlayConfig = {
  modelId: 'IceGirl',
  modelPath: 'C:\\Users\\ADVAN\\Downloads\\IceGirl_Live2d\\IceGIrl Live2D\\IceGirl.model3.json',
  position: { x: 960, y: 540 },
  scale: 0.2,
  lookAtEnabled: true,
  clickThroughLock: false,
  fpsLimit: 60,
  opacity: 1.0,
  alwaysOnTop: true,
};

let app: Application;
let model: Live2DModel | null = null;
let isHovering = false;
let prevHovering = false;
let lastMousePos = { x: 400, y: 400 };
let hitTestInterval: ReturnType<typeof setInterval> | null = null;
let cursorInterval: ReturnType<typeof setInterval> | null = null;

// Cache bounds terakhir yang valid. Saat model invisible (visible=false),
// getBounds() bisa mengembalikan area kosong di sebagian versi PIXI, yang
// akan bikin hover "berkedip". Cache dipakai sebagai fallback.
let cachedBounds = { x: 0, y: 0, width: 0, height: 0 };

// Posisi kursor OS (pixel fisik). Dipolling via Rust karena window overlay
// click-through -> event mousemove tidak selalu tersedia.
let osCursor = { x: 0, y: 0 };

// Saat kursor di atas avatar, avatar jadi transparan penuh (invisible).
// Pakai 0.001 (bukan 0.0) supaya tidak memicu alpha===0 check di pixi-live2d-display
// yang bisa menyebabkan model tidak render sama sekali.
const OVERLAY_HOVER_ALPHA = 0.001;

function osCursorCss(): { x: number; y: number } {
  const d = window.devicePixelRatio || 1;
  return { x: osCursor.x / d, y: osCursor.y / d };
}

const canvas = document.getElementById('canvas') as HTMLCanvasElement;

function toModelUrl(filePath: string): string {
  if (filePath.startsWith('http://') || filePath.startsWith('https://') || filePath.startsWith('live2d://')) {
    return filePath;
  }
  const normalized = filePath.replace(/\\/g, '/');
  return `http://live2d.localhost/${encodeURI(normalized)}`;
}

// Get initialization data from Rust backend
async function getInitData(): Promise<{ modelPath: string; config: OverlayConfig }> {
  try {
    const data = await invoke('get_init_data') as { model_path: string; config: OverlayConfig };
    if (data && data.model_path) {
      return {
        modelPath: data.model_path,
        config: data.config,
      };
    }
  } catch (e) {
    console.warn('Failed to get init data from backend, using defaults:', e);
  }

  return {
    modelPath: config.modelPath,
    config: { ...config },
  };
}

// Initialize PixiJS Application
async function initPixi() {
  live2dConfig.logLevel = live2dConfig.LOG_LEVEL_VERBOSE;
  live2dConfig.motionSync = true;
  live2dConfig.sound = false;

  app = new Application({
    view: canvas,
    width: window.innerWidth,
    height: window.innerHeight,
    backgroundAlpha: 0,
    resolution: window.devicePixelRatio || 1,
    autoDensity: true,
    antialias: true,
    preserveDrawingBuffer: true,
  });

  // Expose untuk debugging via DevTools/CDP.
  (window as any).__overlayApp = app;

  canvas.style.position = 'fixed';
  canvas.style.top = '0';
  canvas.style.left = '0';
  canvas.style.width = '100vw';
  canvas.style.height = '100vh';
  canvas.style.pointerEvents = 'auto';
  canvas.style.zIndex = '999999';
  // Transisi halus untuk hide/show (fade opacity). di-set sekali di sini,
  // lalu di-toggle saat state hover berubah (bukan tiap frame).
  canvas.style.transition = 'opacity 0.3s ease';

  window.addEventListener('resize', () => {
    app.renderer.resize(window.innerWidth, window.innerHeight);
  });

  app.ticker.maxFPS = config.fpsLimit || 60;
  app.ticker.add((deltaTime: number) => {
    onTick(deltaTime);
  });
}

// Terapkan config (satuan pixel fisik layar) ke model. Window overlay
// berjalan di koordinat CSS (physical / devicePixelRatio), jadi posisi & scale
// harus dibagi DPR supaya cocok dengan preview yang memakai pixel fisik.
function applyModelConfig() {
  if (!model) return;
  const d = window.devicePixelRatio || 1;
  model.scale.set(config.scale / d);
  model.position.set(config.position.x / d, config.position.y / d);
  model.anchor.set(0.5, 0.5);
  model.alpha = config.opacity;
  model.visible = true;
}

// Load Live2D Model
async function loadModel(rawPath: string) {
  if (model) {
    app.stage.removeChild(model);
    model.destroy();
    model = null;
  }

  const modelUrl = toModelUrl(rawPath);
  console.log('Loading Live2D model from:', modelUrl);

  try {
    model = await Live2DModel.from(modelUrl, {
      autoInteract: false,
    });

    applyModelConfig();

    app.stage.addChild(model);
    console.log('Live2D model successfully loaded and added to stage!');
  } catch (error) {
    console.error('Failed to load model from', modelUrl, error);
  }
}

// Animation tick
let dbgFrame = 0;
function onTick(_deltaTime: number) {
  if (!model) return;

  // Hover detection via bounding box avatar, pakai posisi kursor OS (CSS).
  // Model selalu visible (render jalan); yang di-fade adalah opacity canvas.
  const vb = model.getBounds();
  if (vb.width > 0 && vb.height > 0) {
    cachedBounds = { x: vb.x, y: vb.y, width: vb.width, height: vb.height };
  }
  const b = cachedBounds;
  const c = osCursorCss();
  const rect = canvas.getBoundingClientRect();
  const mx = c.x - rect.left;
  const my = c.y - rect.top;
  isHovering = b.width > 0 && mx >= b.x && mx <= b.x + b.width && my >= b.y && my <= b.y + b.height;

  // Transisi halus hide/show via CSS opacity pada canvas (bukan model.visible
  // yang hard-toggle). Hanya update saat state hover BERUBAH supaya transisi
  // CSS tidak di-reset tiap frame.
  if (isHovering !== prevHovering) {
    prevHovering = isHovering;
    canvas.style.opacity = isHovering ? '0' : '1';
  }
  // Selaraskan alpha (tidak berpengaruh ke render v0.5, tapi untuk metadata).
  model.alpha = isHovering ? OVERLAY_HOVER_ALPHA : config.opacity;

  if (++dbgFrame % 30 === 0) {
    console.log(
      `[DBG] cur=${osCursor.x},${osCursor.y} css=${Math.round(mx)},${Math.round(my)} ` +
      `hov=${isHovering} op=${canvas.style.opacity} a=${model.alpha.toFixed(2)} ` +
      `bounds=${Math.round(b.x)},${Math.round(b.y)},${Math.round(b.width)},${Math.round(b.height)} ` +
      `pos=${Math.round(model.position.x)},${Math.round(model.position.y)} ` +
      `win=${window.innerWidth}x${window.innerHeight} dpr=${window.devicePixelRatio}`
    );
  }
}

// Smart click-through: check hit area
async function checkHitTest() {
  if (!model || config.clickThroughLock) return;

  try {
    // Click-through berbasis AREA (bounds), bukan alpha pixel. Dengan begitu
    // window tidak lagi jadi click-through saat karakter transparan, sehingga
    // event klik tetap sampai ke canvas (ekspresi bisa dipicu). Window hanya
    // tembus ke desktop saat kursor berada DI LUAR area karakter.
    // Pakai cachedBounds agar konsisten walau model sedang invisible.
    const b = cachedBounds;
    const rect = canvas.getBoundingClientRect();
    const c = osCursorCss();
    const mx = c.x - rect.left;
    const my = c.y - rect.top;

    const inside =
      b.width > 0 &&
      mx >= b.x && mx <= b.x + b.width && my >= b.y && my <= b.y + b.height;

    await invoke('set_ignore_cursor_events', { ignore: !inside });
  } catch (e) {
    // Ignore errors during hit test
  }
}

// Event listeners
function setupEventListeners() {
  window.addEventListener('mousemove', (e) => {
    lastMousePos = { x: e.clientX, y: e.clientY };
  });

  // Drag & zoom DINONAKTIFKAN di overlay. Posisi/scale hanya bisa diatur
  // melalui Master App. Dengan begitu, selama karakter tidak sedang diatur di
  // master, ia tidak pernah "isDragging" -> hover langsung transparan penuh.
  canvas.addEventListener('click', async (e) => {
    if (!model || config.clickThroughLock) return;

    // Trigger random expression or motion on tap
    try {
      const expressions = (model.internalModel as any).expressionManager?.definitions;
      if (expressions && expressions.length > 0) {
        const randomExpr = expressions[Math.floor(Math.random() * expressions.length)];
        const exprName = randomExpr.name || randomExpr.file;
        if (exprName) {
          await model.expression(exprName);
        }
      }
    } catch (e) {
      // Ignore expression failure
    }
  });
}

// Initialize
async function main() {
  const init = await getInitData();
  config = init.config;

  // Pastikan opacity tidak 0. Kalau opacity = 0, overlay lahir invisible dan
  // efek hover-fade tidak akan pernah terlihat (targetAlpha juga 0). Clamp ke
  // rentang aman supaya mascot selalu punya baseline yang terlihat.
  config.opacity = Math.max(0.05, Math.min(1, config.opacity));

  await initPixi();
  await loadModel(init.modelPath);
  setupEventListeners();

  applyModelConfig();

  // Poll hit test at 25 Hz (40ms)
  hitTestInterval = setInterval(checkHitTest, 40);

  // Poll posisi kursor OS (40ms) supaya look-at & hover tetap jalan walau
  // window sedang click-through (mousemove tidak tersedia).
  cursorInterval = setInterval(async () => {
    try {
      const r = await invoke('get_cursor_pos');
      osCursor = { x: r[0], y: r[1] };
    } catch {
      // ignore
    }
  }, 40);

  // Debug hook (DevTools/CDP).
  (window as any).__ovDbg = () => ({
    osCursor: { x: osCursor.x, y: osCursor.y },
    isHovering,
    alpha: model ? model.alpha : null,
    dpr: window.devicePixelRatio,
  });

  console.log('Live2D Overlay initialized successfully');
}

main().catch(console.error);

// Cleanup
window.addEventListener('beforeunload', () => {
  if (hitTestInterval) clearInterval(hitTestInterval);
  if (cursorInterval) clearInterval(cursorInterval);
  if (model) model.destroy();
  if (app) app.destroy();
});
