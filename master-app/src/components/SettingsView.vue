<script setup lang="ts">
import { useSettingsStore } from '@/stores/settings';
import { useOverlayStore } from '@/stores/overlay';
import { useModelStore } from '@/stores/model';
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const settingsStore = useSettingsStore();
const overlayStore = useOverlayStore();
const modelStore = useModelStore();

const testPosition = ref({ x: 100, y: 100 });
const showResetConfirm = ref(false);
const capturingHotkey = ref(false);

const modelsForSelect = computed(() => modelStore.models.map(m => ({ value: m.id, label: m.name })));

async function saveAll() {
  await settingsStore.saveSettings();
  await settingsStore.saveOverlayConfig();
}

async function resetOverlayConfig() {
  showResetConfirm.value = true;
}

async function confirmReset() {
  settingsStore.resetOverlayConfig();
  showResetConfirm.value = false;
}

async function openModelsFolder() {
  try {
    await invoke('open_models_folder');
  } catch (e) {
    console.error('Open folder failed:', e);
  }
}

async function testPositionApply() {
  if (overlayStore.isRunning) {
    settingsStore.updateOverlayConfig('position', testPosition.value);
  }
}

// ===== Push-to-talk hotkey capture =====
function startCaptureHotkey() {
  capturingHotkey.value = true;
}

function finishCapture(hotkey: string) {
  settingsStore.overlayConfig.voice.pttHotkey = hotkey;
  settingsStore.overlayConfig.voice.pttMouseButton = '';
  capturingHotkey.value = false;
  settingsStore.saveOverlayConfig();
}

function handleKeyCapture(e: KeyboardEvent) {
  if (!capturingHotkey.value) return;
  // Tunggu tombol non-modifier.
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return;
  e.preventDefault();
  e.stopPropagation();

  const parts: string[] = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.shiftKey) parts.push('Shift');
  if (e.altKey) parts.push('Alt');
  if (e.metaKey) parts.push('Super');

  let key = e.key;
  if (key === ' ') key = 'Space';
  else if (key === 'Escape') { capturingHotkey.value = false; return; }
  else if (key.length === 1) key = key.toUpperCase();
  parts.push(key);

  finishCapture(parts.join('+'));
}

// Mouse side buttons: 3 = Back (Mouse4 / XBUTTON1), 4 = Forward (Mouse5 / XBUTTON2).
function handleMouseCapture(e: MouseEvent) {
  if (!capturingHotkey.value) return;
  if (e.button === 3 || e.button === 4) {
    e.preventDefault();
    e.stopPropagation();
    const button = e.button === 3 ? 'Mouse4' : 'Mouse5';
    settingsStore.overlayConfig.voice.pttMouseButton = button;
    settingsStore.overlayConfig.voice.pttHotkey = '';
    capturingHotkey.value = false;
    settingsStore.saveOverlayConfig();
  }
}

const pttLabel = computed(() => {
  const mouse = settingsStore.overlayConfig.voice.pttMouseButton;
  if (mouse) return `${mouse} (mouse)`;
  return settingsStore.overlayConfig.voice.pttHotkey || 'Belum diset';
});

onMounted(() => {
  window.addEventListener('keydown', handleKeyCapture, true);
  window.addEventListener('mousedown', handleMouseCapture, true);
});
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyCapture, true);
  window.removeEventListener('mousedown', handleMouseCapture, true);
});
</script>

<template>
  <div class="settings-view">
    <div class="settings-header">
      <h1>Settings</h1>
      <p class="subtitle">Configure overlay behavior and application preferences</p>
    </div>

    <div class="settings-grid">
      <!-- Overlay Config Section -->
      <section class="settings-section">
        <h2 class="section-title">Overlay Configuration</h2>
        
        <div class="setting-group">
          <label class="setting-label">Model</label>
          <select class="setting-select" v-model="settingsStore.overlayConfig.modelId" @change="saveAll" :disabled="overlayStore.isRunning">
            <option value="">Select a model...</option>
            <option v-for="m in modelsForSelect" :key="m.value" :value="m.value">{{ m.label }}</option>
          </select>
        </div>

        <div class="setting-row">
          <div class="setting-group">
            <label class="setting-label">Position X</label>
            <input type="number" class="setting-input" v-model.number="settingsStore.overlayConfig.position.x" @change="saveAll" :disabled="overlayStore.isRunning" />
          </div>
          <div class="setting-group">
            <label class="setting-label">Position Y</label>
            <input type="number" class="setting-input" v-model.number="settingsStore.overlayConfig.position.y" @change="saveAll" :disabled="overlayStore.isRunning" />
          </div>
        </div>

        <div class="setting-group">
          <label class="setting-label">Scale</label>
          <div class="slider-container">
            <input type="range" class="setting-slider" v-model.number="settingsStore.overlayConfig.scale" min="0.1" max="5" step="0.05" @input="saveAll" :disabled="overlayStore.isRunning" />
            <span class="slider-value">{{ settingsStore.overlayConfig.scale.toFixed(2) }}x</span>
          </div>
        </div>

        <div class="setting-group">
          <label class="setting-label">Opacity</label>
          <div class="slider-container">
            <input type="range" class="setting-slider" v-model.number="settingsStore.overlayConfig.opacity" min="0.1" max="1" step="0.05" @input="saveAll" :disabled="overlayStore.isRunning" />
            <span class="slider-value">{{ (settingsStore.overlayConfig.opacity * 100).toFixed(0) }}%</span>
          </div>
        </div>

        <div class="setting-group">
          <label class="setting-label">FPS Limit</label>
          <select class="setting-select" v-model="settingsStore.overlayConfig.fpsLimit" @change="saveAll" :disabled="overlayStore.isRunning">
            <option value="30">30 FPS (Lower CPU)</option>
            <option value="60">60 FPS (Smoother)</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.overlayConfig.lookAtEnabled" @change="saveAll" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Mouse Look-At (eyes follow cursor)</span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.overlayConfig.clickThroughLock" @change="saveAll" :disabled="overlayStore.isRunning" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Click-Through Lock (Ctrl+Shift+L to toggle)</span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.overlayConfig.alwaysOnTop" @change="saveAll" :disabled="overlayStore.isRunning" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Always on Top</span>
          </label>
        </div>

        <div class="setting-actions">
          <button class="btn btn-secondary" @click="resetOverlayConfig">Reset to Defaults</button>
        </div>
      </section>

      <!-- Voice Assistant Section -->
      <section class="settings-section">
        <h2 class="section-title">Voice Assistant (AI)</h2>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.overlayConfig.voice.enabled" @change="saveAll" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Aktifkan Voice Assistant</span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-label">Groq API Key (Whisper STT)</label>
          <input type="password" class="setting-input" v-model="settingsStore.overlayConfig.voice.groqApiKey" @change="saveAll" placeholder="gsk_..." autocomplete="off" />
        </div>

        <div class="setting-group">
          <label class="setting-label">Whisper Model</label>
          <select class="setting-select" v-model="settingsStore.overlayConfig.voice.whisperModel" @change="saveAll">
            <option value="whisper-large-v3">whisper-large-v3 (akurat, default)</option>
            <option value="whisper-large-v3-turbo">whisper-large-v3-turbo (perlu di-enable di Groq)</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-label">LLM Endpoint</label>
          <input type="text" class="setting-input" v-model="settingsStore.overlayConfig.voice.llmEndpoint" @change="saveAll" placeholder="http://localhost:20128/v1" />
        </div>

        <div class="setting-group">
          <label class="setting-label">LLM API Key</label>
          <input type="password" class="setting-input" v-model="settingsStore.overlayConfig.voice.llmApiKey" @change="saveAll" placeholder="sk-..." autocomplete="off" />
        </div>

        <div class="setting-group">
          <label class="setting-label">LLM Model</label>
          <input type="text" class="setting-input" v-model="settingsStore.overlayConfig.voice.llmModel" @change="saveAll" placeholder="claude-mycombo" />
        </div>

        <div class="setting-group">
          <label class="setting-label">System Prompt</label>
          <textarea class="setting-input" rows="2" v-model="settingsStore.overlayConfig.voice.systemPrompt" @change="saveAll"></textarea>
        </div>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.overlayConfig.voice.ttsEnabled" @change="saveAll" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Bacakan balasan (Text-to-Speech)</span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-label">Push-To-Talk Bind</label>
          <button class="btn btn-secondary" @click="startCaptureHotkey">
            {{ capturingHotkey ? 'Tekan tombol keyboard / mouse samping...' : pttLabel }}
          </button>
          <p class="setting-hint">Klik tombol di atas, lalu tekan kombinasi tombol keyboard <b>atau</b> tombol mouse samping (Mouse4/Back atau Mouse5/Forward). Esc untuk batal.</p>
        </div>
      </section>

      <!-- Application Settings Section -->
      <section class="settings-section">
        <h2 class="section-title">Application</h2>
        
        <div class="setting-group">
          <label class="setting-label">Theme</label>
          <select class="setting-select" v-model="settingsStore.settings.theme" @change="saveAll">
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-label">Language</label>
          <select class="setting-select" v-model="settingsStore.settings.language" @change="saveAll">
            <option value="id">Indonesian</option>
            <option value="en">English</option>
          </select>
        </div>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.settings.autoStartOverlay" @change="saveAll" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Auto-start overlay on launch</span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.settings.minimizeToTray" @change="saveAll" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Minimize to system tray</span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-toggle">
            <input type="checkbox" v-model="settingsStore.settings.closeToTray" @change="saveAll" />
            <span class="toggle-slider"></span>
            <span class="toggle-label">Close window minimizes to tray</span>
          </label>
        </div>

        <div class="setting-group">
          <label class="setting-label">Models Folder</label>
          <div class="folder-input">
            <input type="text" class="setting-input" :value="settingsStore.settings.modelsDirectory || '(default)'" readonly />
            <button class="btn btn-secondary" @click="openModelsFolder">Open Folder</button>
          </div>
        </div>
      </section>

      <!-- About Section -->
      <section class="settings-section">
        <h2 class="section-title">About</h2>
        <div class="about-card">
          <div class="about-logo">
            <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
            </svg>
          </div>
          <h3>Live2D Mascot Master</h3>
          <p class="version">Version 1.0.0</p>
          <p class="description">OBS-style control panel for Live2D desktop mascots</p>
          <div class="about-links">
            <a href="#" class="btn btn-ghost btn-sm">Documentation</a>
            <a href="#" class="btn btn-ghost btn-sm">GitHub</a>
          </div>
        </div>
      </section>
    </div>

    <ConfirmDialog
      v-model:show="showResetConfirm"
      title="Reset Overlay Configuration?"
      message="This will reset all overlay settings (position, scale, FPS, etc.) to their default values. This action cannot be undone."
      confirm-text="Reset"
      variant="danger"
      @confirm="confirmReset"
    />
  </div>
</template>

<style scoped>
.settings-view {
  max-width: 800px;
  margin: 0 auto;
}

.settings-header {
  margin-bottom: 24px;
}

.settings-header h1 {
  font-size: 24px;
  font-weight: 700;
  color: var(--fg-primary);
}

.settings-grid {
  display: grid;
  gap: 24px;
}

.settings-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 20px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-secondary);
  margin-bottom: 20px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.setting-group {
  margin-bottom: 16px;
}

.setting-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.setting-label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-secondary);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.setting-input {
  width: 100%;
  padding: 10px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--fg-primary);
  font-size: 13px;
  transition: all var(--transition);
}

textarea.setting-input {
  resize: vertical;
  min-height: 52px;
  font-family: inherit;
}

.setting-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--fg-muted);
}

.setting-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-light);
}

.setting-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.setting-select {
  width: 100%;
  padding: 10px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--fg-primary);
  font-size: 13px;
  cursor: pointer;
  appearance: none;
  transition: all var(--transition);
}

.setting-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-light);
}

.setting-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.setting-slider {
  flex: 1;
  height: 6px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--bg-tertiary);
  border-radius: 3px;
  outline: none;
}

.setting-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  background: var(--accent);
  border-radius: 50%;
  cursor: pointer;
  transition: transform var(--transition);
}

.setting-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
}

.setting-slider::-moz-range-thumb {
  width: 18px;
  height: 18px;
  background: var(--accent);
  border: none;
  border-radius: 50%;
  cursor: pointer;
}

.slider-value {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg-secondary);
  min-width: 50px;
  text-align: right;
  font-family: 'JetBrains Mono', monospace;
}

.setting-toggle {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
}

.setting-toggle input[type="checkbox"] {
  display: none;
}

.toggle-slider {
  width: 44px;
  height: 24px;
  background: var(--bg-tertiary);
  border-radius: 12px;
  position: relative;
  transition: background var(--transition);
  flex-shrink: 0;
}

.setting-toggle input:checked + .toggle-slider {
  background: var(--accent);
}

.toggle-slider::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
  transition: transform var(--transition);
}

.setting-toggle input:checked + .toggle-slider::after {
  transform: translateX(20px);
}

.setting-toggle input:disabled + .toggle-slider {
  opacity: 0.5;
}

.toggle-label {
  font-size: 13px;
  color: var(--fg-primary);
  user-select: none;
}

.folder-input {
  display: flex;
  gap: 8px;
}

.folder-input .setting-input {
  flex: 1;
}

.setting-actions {
  padding-top: 8px;
  border-top: 1px solid var(--border);
  margin-top: 8px;
}

.setting-actions .btn {
  width: 100%;
}

.about-card {
  text-align: center;
  padding: 20px;
}

.about-logo {
  color: var(--accent);
  margin-bottom: 12px;
}

.about-card h3 {
  font-size: 18px;
  font-weight: 600;
  color: var(--fg-primary);
  margin-bottom: 4px;
}

.version {
  font-size: 13px;
  color: var(--fg-muted);
  margin-bottom: 8px;
}

.description {
  font-size: 13px;
  color: var(--fg-secondary);
  margin-bottom: 16px;
}

.about-links {
  display: flex;
  gap: 8px;
  justify-content: center;
}

@media (max-width: 640px) {
  .setting-row {
    grid-template-columns: 1fr;
  }
}
</style>