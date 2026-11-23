import { Application, Ticker } from 'pixi.js';
import { Live2DModel, config as live2dConfig } from 'pixi-live2d-display';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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

interface VoiceConfig {
  enabled: boolean;
  groqApiKey: string;
  whisperModel: string;
  llmEndpoint: string;
  llmApiKey: string;
  llmModel: string;
  systemPrompt: string;
  ttsVoice: string;
  ttsEnabled: boolean;
  pttHotkey: string;
  pttMouseButton: string;
}

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
  voice?: VoiceConfig;
}

const DEFAULT_VOICE: VoiceConfig = {
  enabled: false,
  groqApiKey: '',
  whisperModel: 'whisper-large-v3',
  llmEndpoint: 'http://localhost:20128/v1',
  llmApiKey: '',
  llmModel: 'claude-mycombo',
  systemPrompt: 'Kamu adalah asisten AI yang ramah dan membantu. Jawab singkat dan jelas dalam bahasa Indonesia.',
  ttsVoice: 'id-ID-GadisNeural',
  ttsEnabled: true,
  pttHotkey: 'Ctrl+Shift+Space',
  pttMouseButton: '',
};

let voiceConfig: VoiceConfig = { ...DEFAULT_VOICE };
let lastPttHotkey = '';
let lastPttMouse = '';

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

// Posisi dasar model (CSS px) — dipakai Mood Loop untuk offset "badan mundur"
// saat startled tanpa mengganggu config drag/position.
let basePos = { x: 0, y: 0 };

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
  const px = config.position.x / d;
  const py = config.position.y / d;
  model.position.set(px, py);
  basePos = { x: px, y: py };
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

  // Inisialisasi timing Mood Loop (hindari yawn/ganti mood di frame pertama).
  {
    const bootT = performance.now();
    moodState.lastT = bootT;
    moodState.nextSwitch = bootT + 1500;
    moodState.nextYawn = bootT + 90000;
  }

    app.stage.addChild(model);
    console.log('Live2D model successfully loaded and added to stage!');
  } catch (error) {
    console.error('Failed to load model from', modelUrl, error);
  }
}

// ===== Mood Loop (personality idle, tanpa AI) =====
// Mengubah "mood" karakter tiap 15-30 detik secara acak (Normal 60% /
// Penasaran 20% / Mengantuk 20%), plus reaksi startled saat kursor melesat
// cepat di dekat karakter, dan mode malam (kelopak turun + menguap) setelah
// jam 23:00. Semua via setParameterValueById — TIDAK memakai motion/expr,
// jadi tidak butuh file .motion3/.exp3 yang di-referensikan model3.json.
//
// Catatan: fokus look-at (model.focus) mengontrol ParamAngleX/Y + ParamEyeBallX/Y,
// jadi Mood Loop hanya menyentuh parameter yang TIDAK dikontrol fokus
// (AngleZ, EyeOpen, MouthForm, MouthOpenY, BrowForm) agar tidak saling menimpa.
type Mood = 'normal' | 'curious' | 'sleepy';

interface MoodParams {
  angleZ: number;
  eyeL: number;
  eyeR: number;
  mouthForm: number;
  mouthOpen: number;
  browL: number;
  browR: number;
}

const MOOD_TARGETS: Record<Mood, MoodParams> = {
  normal:  { angleZ: 0,  eyeL: 1.0,  eyeR: 1.0,  mouthForm: 0.3, mouthOpen: 0, browL: 0,    browR: 0 },
  curious: { angleZ: 12, eyeL: 1.0,  eyeR: 1.0,  mouthForm: 0.1, mouthOpen: 0, browL: 0.5,  browR: 0.5 },
  sleepy:  { angleZ: 5,  eyeL: 0.65, eyeR: 0.65, mouthForm: 0.0, mouthOpen: 0, browL: -0.1, browR: -0.1 },
};

const moodState = {
  mood: 'normal' as Mood,
  nextSwitch: 0,
  cur: { angleZ: 0, eyeL: 1, eyeR: 1, mouthForm: 0.3, mouthOpen: 0, browL: 0, browR: 0 } as MoodParams,
  lastT: 0,
  startleUntil: 0,
  bodyOff: 0,
  yawning: false,
  yawnStart: 0,
  nextYawn: 0,
};

function pickMood(): Mood {
  const r = Math.random();
  if (r < 0.6) return 'normal';
  if (r < 0.8) return 'curious';
  return 'sleepy';
}

function isNightNow(): boolean {
  const h = new Date().getHours();
  return h >= 23 || h < 5;
}

// Picu ekspresi kaget 1.5 detik (dipanggil saat kursor melesat cepat dekat avatar).
function triggerStartle() {
  const now = performance.now();
  if (now < moodState.startleUntil) return; // sudah kaget
  moodState.startleUntil = now + 1500;
}

function coreModel(): any {
  return (model as any)?.internalModel?.coreModel ?? null;
}

function applyMoodParams(c: MoodParams) {
  const core = coreModel();
  if (!core) return;
  const set = (id: string, v: number) => {
    try { core.setParameterValueById(id, v); } catch { /* param tidak ada */ }
  };
  set('ParamAngleZ', c.angleZ);
  set('ParamEyeLOpen', c.eyeL);
  set('ParamEyeROpen', c.eyeR);
  set('ParamMouthForm', c.mouthForm);
  set('ParamMouthOpenY', c.mouthOpen);
  set('ParamBrowLForm', c.browL);
  set('ParamBrowRForm', c.browR);
}

function updateMood(now: number) {
  if (!model) return;
  const dt = moodState.lastT ? Math.min(0.1, (now - moodState.lastT) / 1000) : 0.016;
  moodState.lastT = now;

  // Ganti mood tiap 15-30 detik.
  if (now >= moodState.nextSwitch) {
    moodState.mood = pickMood();
    moodState.nextSwitch = now + (15000 + Math.random() * 15000);
  }

  const night = isNightNow();
  const tgt: MoodParams = { ...MOOD_TARGETS[moodState.mood] };

  // Mode malam: kelopak mata sedikit turun.
  if (night) {
    tgt.eyeL = Math.min(tgt.eyeL, 0.68);
    tgt.eyeR = Math.min(tgt.eyeR, 0.68);
  }

  // Ayunan kepala pelan (napas); lebih lambat & besar saat mengantuk.
  const tSec = now / 1000;
  const swaySpeed = moodState.mood === 'sleepy' ? 0.6 : 1.2;
  const swayAmp = moodState.mood === 'sleepy' ? 4 : (moodState.mood === 'curious' ? 1.5 : 2.5);
  tgt.angleZ += Math.sin(tSec * swaySpeed) * swayAmp;

  // Startled (1.5 dtk): alis 1.0, mata melebar 1.2, mulut sedikit terbuka.
  const startled = now < moodState.startleUntil;
  if (startled) {
    tgt.browL = 1.0; tgt.browR = 1.0;
    tgt.eyeL = 1.2; tgt.eyeR = 1.2;
    tgt.mouthOpen = Math.max(tgt.mouthOpen, 0.25);
  }

  // Menguap (mode malam, tiap 3-6 menit): mulut buka-tutup pelan + mata menyipit.
  if (night && !moodState.yawning && now >= moodState.nextYawn) {
    moodState.yawning = true;
    moodState.yawnStart = now;
    moodState.nextYawn = now + (180000 + Math.random() * 180000);
  }
  if (moodState.yawning) {
    const p = (now - moodState.yawnStart) / 3500; // 3.5 dtk
    if (p >= 1) {
      moodState.yawning = false;
    } else if (!startled) {
      const open = Math.sin(p * Math.PI); // 0 -> 1 -> 0
      tgt.mouthOpen = Math.max(tgt.mouthOpen, open * 0.9);
      const squint = 1 - open * 0.6;
      tgt.eyeL = Math.min(tgt.eyeL, squint);
      tgt.eyeR = Math.min(tgt.eyeR, squint);
      tgt.browL = Math.min(tgt.browL, -0.2);
      tgt.browR = Math.min(tgt.browR, -0.2);
    }
  }

  // Lerp halus current -> target.
  const k = 1 - Math.exp(-dt * 4);
  const c = moodState.cur;
  c.angleZ  += (tgt.angleZ  - c.angleZ)  * k;
  c.eyeL    += (tgt.eyeL    - c.eyeL)    * k;
  c.eyeR    += (tgt.eyeR    - c.eyeR)    * k;
  c.mouthForm += (tgt.mouthForm - c.mouthForm) * k;
  c.mouthOpen += (tgt.mouthOpen - c.mouthOpen) * k;
  c.browL   += (tgt.browL   - c.browL)   * k;
  c.browR   += (tgt.browR   - c.browR)   * k;

  applyMoodParams(c);

  // "Badan sedikit mundur" saat startled: offset posisi kecil, kembali halus.
  const targetOff = startled ? 10 : 0;
  moodState.bodyOff += (targetOff - moodState.bodyOff) * 0.25;
  model.position.set(basePos.x, basePos.y + moodState.bodyOff);
}

// Deteksi kursor melesat cepat dekat area karakter -> startled.
function maybeStartleBySpeed(speedPxPerSec: number) {
  if (speedPxPerSec < 1400) return; // ambang "melesat sangat cepat"
  const b = cachedBounds;
  if (b.width <= 0) return;
  const c = osCursorCss();
  const rect = canvas.getBoundingClientRect();
  const mx = c.x - rect.left;
  const my = c.y - rect.top;
  const margin = 140;
  const near =
    mx >= b.x - margin && mx <= b.x + b.width + margin &&
    my >= b.y - margin && my <= b.y + b.height + margin;
  if (near) triggerStartle();
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

  // Look-at: arahkan pandangan model ke posisi kursor (bila diaktifkan).
  // Fokus pakai koordinat layar (mx,my); pixi-live2d-display menangani transform.
  if (config.lookAtEnabled) {
    model.focus(mx, my);
  } else {
    // Look-at OFF -> reset fokus ke netral (0,0) supaya karakter langsung
    // hadap depan, bukan menatap posisi kursor terakhir.
    (model.internalModel as any).focusController?.focus(0, 0, true);
  }

  // Mood Loop: ubah ekspresi/idle tiap 15-30 dtk + reaksi startled + mode malam.
  updateMood(performance.now());

  if (++dbgFrame % 30 === 0) {
    console.log(
      `[DBG] cur=${osCursor.x},${osCursor.y} css=${Math.round(mx)},${Math.round(my)} ` +
      `hov=${isHovering} op=${canvas.style.opacity} a=${model.alpha.toFixed(2)} ` +
      `bounds=${Math.round(b.x)},${Math.round(b.y)},${Math.round(b.width)},${Math.round(b.height)} ` +
      `pos=${Math.round(model.position.x)},${Math.round(model.position.y)} ` +
      `win=${window.innerWidth}x${window.innerHeight} dpr=${window.devicePixelRatio} mood=${moodState.mood}`
    );
  }
}

// Click-through: window SELALU tembus ke app di belakang karakter, supaya
// app di belakang mascot tetap bisa diklik. Interaksi (ekspresi) dipicu via
// global hotkey Ctrl+Shift+E, bukan klik mouse.
async function checkHitTest() {
  if (config.clickThroughLock) return; // lock = biarkan window menangkap klik
  try {
    await invoke('set_ignore_cursor_events', { ignore: true });
  } catch (e) {
    // Ignore errors during hit test
  }
}

// Picu ekspresi / motion acak pada model.
async function triggerRandomExpression() {
  if (!model) return;
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
}

// ===== Voice Assistant =====
let mediaRecorder: MediaRecorder | null = null;
let recordChunks: Blob[] = [];
let isRecording = false;
let chatHistory: { role: string; content: string }[] = [];

function showBubble(text: string) {
  const bubble = document.getElementById('voice-bubble') as HTMLElement | null;
  const textEl = document.getElementById('voice-bubble-text');
  if (!bubble || !textEl) return;
  textEl.textContent = text;
  bubble.style.display = 'block';
  const anyShow = showBubble as any;
  clearTimeout(anyShow._t);
  anyShow._t = setTimeout(() => { bubble.style.display = 'none'; }, 9000);
}

function speak(text: string) {
  if (!voiceConfig.ttsEnabled || !text) return;
  try {
    const u = new SpeechSynthesisUtterance(text);
    u.lang = 'id-ID';
    window.speechSynthesis.cancel();
    window.speechSynthesis.speak(u);
  } catch (e) {
    // ignore
  }
}

async function startRecording() {
  if (isRecording || !voiceConfig.enabled) return;
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    recordChunks = [];
    mediaRecorder = new MediaRecorder(stream);
    mediaRecorder.ondataavailable = (e) => {
      if (e.data.size > 0) recordChunks.push(e.data);
    };
    mediaRecorder.start();
    isRecording = true;
    console.log('recording started');
  } catch (e) {
    console.error('mic error', e);
  }
}

async function stopRecordingAndProcess() {
  if (!isRecording || !mediaRecorder) return;
  isRecording = false;
  const rec = mediaRecorder;
  mediaRecorder = null;
  return new Promise<void>((resolve) => {
    rec.onstop = async () => {
      const blob = new Blob(recordChunks, { type: 'audio/webm' });
      rec.stream.getTracks().forEach((t) => t.stop());
      await processAudio(blob);
      resolve();
    };
    rec.stop();
  });
}

async function processAudio(blob: Blob) {
  if (!voiceConfig.groqApiKey) {
    console.warn('voice: no groq api key');
    showBubble('⚠ Groq API key belum diisi di Settings');
    return;
  }
  if (blob.size < 1000) {
    console.log('voice: audio too short');
    return;
  }
  try {
    const bytes = new Uint8Array(await blob.arrayBuffer());
    const text = (await invoke('transcribe_audio', {
      audio: Array.from(bytes),
      apiKey: voiceConfig.groqApiKey,
      model: voiceConfig.whisperModel,
    })) as string;
    if (!text) {
      console.log('voice: empty transcription');
      showBubble('⚠ Transkripsi kosong (audio tidak jelas?)');
      return;
    }
    console.log('voice transcribed:', text);

    chatHistory.push({ role: 'user', content: text });
    const messages = [{ role: 'system', content: voiceConfig.systemPrompt }, ...chatHistory];

    const reply = (await invoke('chat_completion', {
      messagesJson: JSON.stringify(messages),
      endpoint: voiceConfig.llmEndpoint,
      apiKey: voiceConfig.llmApiKey,
      model: voiceConfig.llmModel,
    })) as string;

    if (reply) {
      chatHistory.push({ role: 'assistant', content: reply });
      if (chatHistory.length > 20) chatHistory = chatHistory.slice(-20);
      showBubble(reply);
      speak(reply);
    }
  } catch (e) {
    // Surface Groq/STT error ke user (bukan cuma console) supaya tidak "diam".
    console.error('voice pipeline error', e);
    const msg = e instanceof Error ? e.message : String(e);
    showBubble(`⚠ Voice error: ${msg}`);
  }
}

// Daftarkan binding push-to-talk. Tombol mouse samping (pttMouseButton) dan
// hotkey keyboard (pttHotkey) bersifat mutually-exclusive — hanya satu yang aktif.
// tauri-plugin-global-shortcut tidak mendukung tombol mouse, jadi tombol mouse
// ditangani oleh low-level mouse hook di Rust (set_mouse_ptt).
function applyPttBinding() {
  const mouse = (voiceConfig.pttMouseButton || '').trim();
  const hotkey = (voiceConfig.pttHotkey || '').trim();

  if (mouse && mouse !== lastPttMouse) {
    lastPttMouse = mouse;
    invoke('set_mouse_ptt', { button: mouse }).catch((e) =>
      console.error('set_mouse_ptt error', e));
  } else if (!mouse && lastPttMouse) {
    lastPttMouse = '';
    invoke('set_mouse_ptt', { button: '' }).catch((e) =>
      console.error('set_mouse_ptt disable error', e));
  }

  // Keyboard hanya dipakai bila tidak ada tombol mouse yang dipilih.
  if (!mouse && hotkey !== lastPttHotkey) {
    lastPttHotkey = hotkey;
    invoke('set_ptt_shortcut', { hotkey }).catch((e) =>
      console.error('set_ptt_shortcut error', e));
  } else if (mouse && lastPttHotkey) {
    // Nonaktifkan shortcut keyboard supaya tidak dobel dengan tombol mouse.
    lastPttHotkey = '';
    invoke('set_ptt_shortcut', { hotkey: '' }).catch((e) =>
      console.error('set_ptt_shortcut clear error', e));
  }
}

// Event listeners
function setupEventListeners() {
  window.addEventListener('mousemove', (e) => {
    lastMousePos = { x: e.clientX, y: e.clientY };
  });

  // Ekspresi dipicu via global hotkey (Ctrl+Shift+E) dari Rust. Klik mouse
  // tidak dipakai karena window click-through (menembus ke app di belakang).
  listen('trigger-expression', () => {
    triggerRandomExpression();
  });

  // Push-to-talk (Ctrl+Shift+Space) dari Rust.
  listen('ptt-down', () => {
    startRecording();
  });
  listen('ptt-up', () => {
    stopRecordingAndProcess();
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
  // window sedang click-through (mousemove tidak tersedia). Sekaligus hitung
  // kecepatan kursor untuk memicu reaksi startled saat melesat cepat.
  let lastCursor = { x: osCursor.x, y: osCursor.y, t: performance.now() };
  cursorInterval = setInterval(async () => {
    try {
      const r = await invoke('get_cursor_pos');
      const nx = r[0];
      const ny = r[1];
      const now = performance.now();
      const dtSec = Math.max(1, now - lastCursor.t) / 1000;
      const dx = nx - lastCursor.x;
      const dy = ny - lastCursor.y;
      const speed = Math.sqrt(dx * dx + dy * dy) / dtSec; // px/detik (fisik)
      osCursor = { x: nx, y: ny };
      lastCursor = { x: nx, y: ny, t: now };
      maybeStartleBySpeed(speed);
    } catch {
      // ignore
    }
  }, 40);

  // Poll runtime config dari Master (500ms) untuk live-update lookAtEnabled
  // (dan field lain) tanpa perlu restart overlay.
  setInterval(async () => {
    try {
      const raw = await invoke('get_runtime_config') as string;
      if (raw) {
        const rc = JSON.parse(raw);
        if (typeof rc.lookAtEnabled === 'boolean') {
          config.lookAtEnabled = rc.lookAtEnabled;
        }
        if (typeof rc.opacity === 'number') {
          config.opacity = Math.max(0.05, Math.min(1, rc.opacity));
        }
        if (rc.voice && typeof rc.voice === 'object') {
          voiceConfig = { ...DEFAULT_VOICE, ...rc.voice };
          applyPttBinding();
        }
      }
    } catch {
      // ignore
    }
  }, 500);

  // Daftarkan binding PTT awal (keyboard hotkey atau tombol mouse samping).
  applyPttBinding();

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
