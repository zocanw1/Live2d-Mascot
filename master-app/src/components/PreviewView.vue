<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { Application, Ticker } from 'pixi.js';
import { Live2DModel, config as live2dConfig } from 'pixi-live2d-display';
import { invoke } from '@tauri-apps/api/core';
import { useModelStore } from '@/stores/model';
import { useSettingsStore } from '@/stores/settings';

const modelStore = useModelStore();
const settingsStore = useSettingsStore();

Live2DModel.registerTicker(Ticker);

const stageEl = ref<HTMLDivElement | null>(null);
const canvasEl = ref<HTMLCanvasElement | null>(null);

let app: Application | null = null;
let model: Live2DModel | null = null;
let dragging = false;
let hovering = false;
let lastPointer = { x: 0, y: 0 };
let resizeObserver: ResizeObserver | null = null;

// Saat kursor di atas avatar, avatar memudar (transparan) supaya layar di
// belakangnya terlihat untuk mengatur posisi.
const HOVER_ALPHA = 0.0;
const hoverInvisible = ref(true);

// Dimensi "screen space" (physical px monitor) — diambil dari hasil capture,
// tapi bisa dioverride manual lewat input Resolution.
const monitor = reactive({ width: 1920, height: 1080 });

// Screenshot desktop sebagai background preview.
const screenshot = ref<string | null>(null);
const capturing = ref(false);
const loadError = ref<string | null>(null);

// Posisi/scale/opacity yang sedang diatur (local; disimpan saat klik Save).
const form = reactive({ x: 400, y: 300, scale: 0.2, opacity: 1 });

const stageStyle = computed(() => ({
  aspectRatio: `${monitor.width} / ${monitor.height}`,
}));

const selectedModel = computed(() => modelStore.selectedModel);

function toModelUrl(filePath: string): string {
  if (filePath.startsWith('http://') || filePath.startsWith('https://') || filePath.startsWith('live2d://')) {
    return filePath;
  }
  const normalized = filePath.replace(/\\/g, '/');
  return `http://live2d.localhost/${encodeURI(normalized)}`;
}

function stageFactor(): number {
  const w = stageEl.value?.clientWidth || 0;
  if (w <= 0) return 1;
  return w / monitor.width;
}

function applyTransform() {
  if (!model) return;
  const f = stageFactor();
  model.scale.set(form.scale * f);
  model.position.set(form.x * f, form.y * f);
}

function updateHover(e: PointerEvent) {
  if (!model || !canvasEl.value) {
    hovering = false;
    return;
  }
  const b = model.getBounds();
  const rect = canvasEl.value.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;
  hovering = mx >= b.x && mx <= b.x + b.width && my >= b.y && my <= b.y + b.height;
}

function resizeApp() {
  if (!app || !stageEl.value) return;
  const w = stageEl.value.clientWidth;
  const h = stageEl.value.clientHeight;
  if (w <= 0 || h <= 0) return;
  app.renderer.resize(w, h);
  applyTransform();
}

// Untuk debugging via DevTools/CDP (tidak mengganggu UI).
function exposeApp() {
  if (app) (window as any).__previewApp = app;
}

async function initPixi() {
  if (app || !canvasEl.value) return;
  live2dConfig.logLevel = live2dConfig.LOG_LEVEL_VERBOSE;
  live2dConfig.motionSync = true;
  live2dConfig.sound = false;

  app = new Application({
    view: canvasEl.value,
    width: stageEl.value?.clientWidth || 800,
    height: stageEl.value?.clientHeight || 450,
    backgroundAlpha: 0,
    resolution: window.devicePixelRatio || 1,
    autoDensity: true,
    antialias: true,
  });

  // Nonaktifkan event handling bawaan PixiJS supaya tidak menyerap/preventDefault
  // event pointer (biar drag kita jalan mulus).
  app.stage.eventMode = 'none';
  const events = (app.renderer as any).events;
  if (events) events.autoPreventDefault = false;

  exposeApp();

  app.ticker.maxFPS = 60;
  app.ticker.add(() => {
    if (!model) return;
    // hoverInvisible: saat kursor di atas avatar (tanpa drag) -> transparan.
    let target = form.opacity;
    if (hoverInvisible.value && hovering && !dragging) {
      target = HOVER_ALPHA;
    }
    model.alpha += (target - model.alpha) * 0.2;
  });

  resizeObserver = new ResizeObserver(() => resizeApp());
  if (stageEl.value) resizeObserver.observe(stageEl.value);

  // Drag: pointerdown di canvas, pointermove/up di window (capture) agar tetap
  // jalan walau kursor keluar area canvas.
  canvasEl.value.addEventListener('pointerdown', onPointerDown);
  window.addEventListener('pointermove', onPointerMove, true);
  window.addEventListener('pointerup', onPointerUp, true);
}

async function loadPreviewModel() {
  loadError.value = null;
  if (!selectedModel.value?.path) {
    if (model) {
      app?.stage.removeChild(model);
      model.destroy();
      model = null;
    }
    return;
  }
  if (!app) await initPixi();
  if (!app) return;

  if (model) {
    app.stage.removeChild(model);
    model.destroy();
    model = null;
  }

  try {
    const url = toModelUrl(selectedModel.value.path);
    model = await Live2DModel.from(url, { autoInteract: false });
    model.anchor.set(0.5, 0.5);
    model.alpha = form.opacity;
    app.stage.addChild(model);
    applyTransform();
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
    console.error('Preview load model failed:', e);
  }
}

function onPointerDown(e: PointerEvent) {
  if (!model) return;
  if (e.button !== 0 && e.pointerType === 'mouse') return;
  dragging = true;
  lastPointer = { x: e.clientX, y: e.clientY };
}

function onPointerMove(e: PointerEvent) {
  if (dragging && model) {
    const dx = e.clientX - lastPointer.x;
    const dy = e.clientY - lastPointer.y;
    lastPointer = { x: e.clientX, y: e.clientY };
    const f = stageFactor();
    form.x += dx / f;
    form.y += dy / f;
    applyTransform();
  }
  updateHover(e);
}

function onPointerUp() {
  dragging = false;
}

function onWheel(e: WheelEvent) {
  if (!model) return;
  e.preventDefault();
  const zoom = e.deltaY > 0 ? 0.9 : 1.1;
  form.scale = Math.max(0.05, Math.min(3, form.scale * zoom));
  applyTransform();
}

async function captureScreen() {
  if (capturing.value) return;
  capturing.value = true;
  try {
    const res = await invoke<{ dataUrl: string; width: number; height: number }>('capture_screen');
    screenshot.value = res.dataUrl;
    monitor.width = res.width;
    monitor.height = res.height;
    await nextTick();
    resizeApp();
  } catch (e) {
    console.error('capture_screen failed:', e);
    loadError.value = e instanceof Error ? e.message : String(e);
  } finally {
    capturing.value = false;
  }
}

function saveConfig() {
  const cfg = settingsStore.overlayConfig;
  cfg.position = { x: Math.round(form.x), y: Math.round(form.y) };
  cfg.scale = form.scale;
  cfg.opacity = form.opacity;
  settingsStore.saveOverlayConfig();
}

function resetPosition() {
  form.x = monitor.width / 2;
  form.y = monitor.height / 2;
  form.scale = 0.2;
  form.opacity = 1;
  applyTransform();
}

// Fit: buat avatar pas ~85% tinggi layar & di tengah.
function fitModel() {
  if (!model) return;
  const b = model.getBounds();
  if (b.height <= 0 || model.scale.x <= 0) return;
  const nativeHeight = b.height / model.scale.x; // tinggi asli model (px)
  if (nativeHeight <= 0) return;
  form.scale = Math.max(0.05, Math.min(3, (monitor.height * 0.85) / nativeHeight));
  form.x = monitor.width / 2;
  form.y = monitor.height / 2;
  applyTransform();
}

watch(
  () => selectedModel.value?.id,
  () => loadPreviewModel()
);

onMounted(async () => {
  // Muat config tersimpan ke form.
  const cfg = settingsStore.overlayConfig;
  form.x = cfg.position?.x ?? monitor.width / 2;
  form.y = cfg.position?.y ?? monitor.height / 2;
  // Kalau masih default pabrikan (100,100), letakkan di tengah layar.
  if (form.x === 100 && form.y === 100) {
    form.x = monitor.width / 2;
    form.y = monitor.height / 2;
  }
  form.scale = cfg.scale ?? 0.2;
  form.opacity = cfg.opacity ?? 1;

  await nextTick();
  await initPixi();
  await loadPreviewModel();
  resizeApp();

  canvasEl.value?.addEventListener('wheel', onWheel, { passive: false });
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
  canvasEl.value?.removeEventListener('pointerdown', onPointerDown);
  window.removeEventListener('pointermove', onPointerMove, true);
  window.removeEventListener('pointerup', onPointerUp, true);
  canvasEl.value?.removeEventListener('wheel', onWheel);
  if (model) {
    model.destroy();
    model = null;
  }
  if (app) {
    app.destroy(false, { children: true });
    app = null;
  }
});
</script>

<template>
  <div class="preview-view">
    <div class="view-header">
      <div>
        <h1>Preview / Atur Posisi</h1>
        <p class="subtitle">Atur posisi & ukuran Live2D di atas layar seperti OBS</p>
      </div>
      <div class="header-actions">
        <select v-model="modelStore.selectedModelId" class="select">
          <option v-for="m in modelStore.models" :key="m.id" :value="m.id">
            {{ m.name }}
          </option>
        </select>
        <button class="btn btn-secondary" @click="captureScreen" :disabled="capturing">
          {{ capturing ? 'Capturing...' : 'Capture Screen' }}
        </button>
      </div>
    </div>

    <div v-if="loadError" class="error-banner">{{ loadError }}</div>

    <div class="editor">
      <div class="stage-wrap">
        <div ref="stageEl" class="stage" :style="stageStyle">
          <img v-if="screenshot" :src="screenshot" class="stage-bg" alt="screen" />
          <div v-else class="stage-placeholder">Capture Screen dulu untuk melihat desktop</div>
          <canvas ref="canvasEl" class="stage-canvas"></canvas>
        </div>
        <p class="hint">
          Drag avatar untuk geser · Scroll untuk zoom · Hover avatar = transparan (lihat layar di belakang)
        </p>
      </div>

      <aside class="controls">
        <div class="control-group">
          <label>Resolution (px)
            <button class="link-btn" @click="captureScreen" :disabled="capturing">auto</button>
          </label>
          <div class="res-row">
            <input type="number" class="num" min="320" max="7680" v-model.number="monitor.width" @change="resizeApp" />
            <span class="x">×</span>
            <input type="number" class="num" min="240" max="4320" v-model.number="monitor.height" @change="resizeApp" />
          </div>
        </div>
        <div class="control-group">
          <label class="check">
            <input type="checkbox" v-model="hoverInvisible" />
            Hover avatar = transparan
          </label>
        </div>
        <div class="control-group">
          <label>Position X
            <input type="number" class="num" v-model.number="form.x" @change="applyTransform" />
          </label>
          <input type="range" min="0" :max="monitor.width" v-model.number="form.x" @input="applyTransform" />
        </div>
        <div class="control-group">
          <label>Position Y
            <input type="number" class="num" v-model.number="form.y" @change="applyTransform" />
          </label>
          <input type="range" min="0" :max="monitor.height" v-model.number="form.y" @input="applyTransform" />
        </div>
        <div class="control-group">
          <label>Scale
            <input type="number" class="num" min="0.05" max="3" step="0.01" v-model.number="form.scale" @change="applyTransform" />
          </label>
          <input type="range" min="0.05" max="2" step="0.01" v-model.number="form.scale" @input="applyTransform" />
        </div>
        <div class="control-group">
          <label>Opacity <span class="val">{{ Math.round(form.opacity * 100) }}%</span></label>
          <input type="range" min="0.1" max="1" step="0.05" v-model.number="form.opacity" @input="applyTransform" />
        </div>

        <div class="btn-row">
          <button class="btn btn-primary" @click="saveConfig">Simpan Posisi</button>
          <button class="btn btn-secondary" @click="fitModel">Fit</button>
          <button class="btn btn-secondary" @click="resetPosition">Reset</button>
        </div>
        <p class="hint small">
          Layar: {{ monitor.width }}×{{ monitor.height }} · Model:
          {{ selectedModel?.name || 'belum dipilih' }}
        </p>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.preview-view {
  max-width: 1100px;
  margin: 0 auto;
}

.view-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.view-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--fg-primary);
}

.subtitle {
  font-size: 13px;
  color: var(--fg-secondary);
  margin-top: 4px;
}

.header-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.select {
  padding: 8px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--fg-primary);
  font-size: 13px;
  min-width: 160px;
}

.editor {
  display: grid;
  grid-template-columns: 1fr 280px;
  gap: 16px;
  align-items: start;
}

.stage-wrap {
  min-width: 0;
}

.stage {
  position: relative;
  width: 100%;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.stage-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  user-select: none;
  pointer-events: none;
}

.stage-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-muted);
  font-size: 14px;
}

.stage-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  touch-action: none;
  cursor: grab;
}

.stage-canvas:active {
  cursor: grabbing;
}

.hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--fg-muted);
}

.hint.small {
  font-size: 11px;
  margin-top: 14px;
}

.controls {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 16px;
}

.control-group {
  margin-bottom: 14px;
}

.control-group label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: var(--fg-secondary);
  margin-bottom: 6px;
}

.control-group .val {
  font-family: 'JetBrains Mono', monospace;
  color: var(--fg-primary);
}

.control-group .num {
  width: 84px;
  padding: 4px 6px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--fg-primary);
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
  text-align: right;
}

.res-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.res-row .num {
  flex: 1;
  width: auto;
  text-align: center;
}

.res-row .x {
  color: var(--fg-muted);
  font-size: 12px;
}

.link-btn {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 11px;
  cursor: pointer;
  padding: 0;
  text-decoration: underline;
}

.control-group label.check {
  display: flex;
  justify-content: flex-start;
  gap: 8px;
  cursor: pointer;
}

.control-group label.check input[type='checkbox'] {
  width: auto;
  accent-color: var(--accent);
}

.control-group input[type='range'] {
  width: 100%;
}

.btn-row {
  display: flex;
  gap: 8px;
  margin-top: 18px;
}

.btn-row .btn {
  flex: 1;
}

.error-banner {
  background: rgba(220, 60, 60, 0.12);
  border: 1px solid rgba(220, 60, 60, 0.4);
  color: var(--destructive, #e05555);
  padding: 10px 14px;
  border-radius: var(--radius-md);
  margin-bottom: 16px;
  font-size: 13px;
}

@media (max-width: 900px) {
  .editor {
    grid-template-columns: 1fr;
  }
}
</style>