<script setup lang="ts">
import { useOverlayStore } from '@/stores/overlay';
import { useModelStore } from '@/stores/model';
import { useSettingsStore } from '@/stores/settings';
import { useRoute } from 'vue-router';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import ConfirmDialog from './ConfirmDialog.vue';

const overlayStore = useOverlayStore();
const modelStore = useModelStore();
const settingsStore = useSettingsStore();
const route = useRoute();

const showStartConfirm = ref(false);
const startButtonLoading = ref(false);

const canStart = computed(() => modelStore.selectedModel && !overlayStore.isRunning);

const pageTitle = computed(() => {
  switch (route.path) {
    case '/models': return 'Models';
    case '/settings': return 'Settings';
    default: return 'Dashboard';
  }
});

const subTitle = computed(() => {
  switch (route.path) {
    case '/models': return 'Manage your Live2D models';
    case '/settings': return 'Configure overlay behavior';
    default: return 'Overview and quick actions';
  }
});

async function handleStart() {
  if (!canStart.value || startButtonLoading.value) return;
  showStartConfirm.value = true;
}

async function confirmStart() {
  if (!modelStore.selectedModel) return;
  showStartConfirm.value = false;
  startButtonLoading.value = true;
  try {
    const config = settingsStore.overlayConfig;
    config.modelId = modelStore.selectedModel.id;
    config.modelPath = modelStore.selectedModel.path;
    await settingsStore.saveOverlayConfig();
    await overlayStore.startOverlay(modelStore.selectedModel.path, config);
  } catch (e) {
    console.error('Start overlay failed:', e);
  } finally {
    startButtonLoading.value = false;
  }
}

async function handleStop() {
  if (!overlayStore.isRunning) return;
  try {
    await overlayStore.stopOverlay();
  } catch (e) {
    console.error('Stop overlay failed:', e);
  }
}

async function handleRestart() {
  if (!modelStore.selectedModel || !overlayStore.isRunning) return;
  startButtonLoading.value = true;
  try {
    const config = settingsStore.overlayConfig;
    config.modelId = modelStore.selectedModel.id;
    config.modelPath = modelStore.selectedModel.path;
    await settingsStore.saveOverlayConfig();
    await overlayStore.restartOverlay(modelStore.selectedModel.path, config);
  } catch (e) {
    console.error('Restart overlay failed:', e);
  } finally {
    startButtonLoading.value = false;
  }
}

async function minimizeToTray() {
  try {
    await invoke('minimize_to_tray');
  } catch (e) {
    console.error('Minimize to tray failed:', e);
  }
}

const playIcon = `<svg viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>`;
const stopIcon = `<svg viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12"/></svg>`;
const refreshIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6"/><path d="M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>`;
const minimizeIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg>`;
const spinnerIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spin"><circle cx="12" cy="12" r="10" stroke-opacity="0.25"/><path d="M12 2a10 10 0 0 1 10 10"/></svg>`;
</script>

<template>
  <header class="header" role="banner">
    <div class="header-left">
      <h1 class="page-title">{{ pageTitle }}</h1>
      <span class="breadcrumb" v-if="subTitle">{{ subTitle }}</span>
    </div>

    <div class="header-center">
      <div class="overlay-controls" v-if="!overlayStore.isRunning">
        <button
          class="btn btn-primary btn-lg"
          @click="handleStart"
          :disabled="!canStart || startButtonLoading"
          :aria-busy="startButtonLoading"
        >
          <span class="btn-icon" v-html="startButtonLoading ? spinnerIcon : playIcon" />
          <span>{{ startButtonLoading ? 'Starting...' : 'Start Overlay' }}</span>
        </button>
        <span class="start-hint" v-if="!modelStore.selectedModel">Select a model first</span>
      </div>

      <div class="overlay-controls" v-else>
        <button
          class="btn btn-secondary"
          @click="handleRestart"
          :disabled="startButtonLoading"
          title="Restart with current settings"
        >
          <span class="btn-icon" v-html="refreshIcon" />
          <span>Restart</span>
        </button>
        <button
          class="btn btn-danger"
          @click="handleStop"
          :disabled="overlayStore.isStopping"
        >
          <span class="btn-icon" v-html="stopIcon" />
          <span>Stop</span>
        </button>
        <span class="status-badge running">
          <span class="status-dot"></span>
          LIVE
        </span>
      </div>
    </div>

    <div class="header-right">
      <button class="btn btn-ghost" @click="minimizeToTray" title="Minimize to tray">
        <span class="btn-icon" v-html="minimizeIcon" />
      </button>
      <div class="user-menu">
        <button class="btn btn-ghost avatar" aria-label="Menu">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
          </svg>
        </button>
      </div>
    </div>

    <ConfirmDialog
      v-model:show="showStartConfirm"
      title="Start Overlay?"
      message="This will launch the Live2D overlay on your desktop with the selected model and settings."
      confirm-text="Start"
      @confirm="confirmStart"
    />
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--header-height);
  padding: 0 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.page-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--fg-primary);
}

.breadcrumb {
  font-size: 12px;
  color: var(--fg-muted);
}

.header-center {
  display: flex;
  align-items: center;
  gap: 12px;
}

.overlay-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.start-hint {
  font-size: 12px;
  color: var(--fg-muted);
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(78, 201, 176, 0.15);
  border: 1px solid var(--success);
  border-radius: var(--radius-md);
  font-size: 11px;
  font-weight: 600;
  color: var(--success);
}

.status-dot {
  width: 6px;
  height: 6px;
  background: var(--success);
  border-radius: 50%;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-menu {
  position: relative;
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-secondary);
  background: var(--bg-tertiary);
}

.avatar:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.btn-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.btn-icon.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.btn-lg {
  padding: 10px 20px;
  font-size: 14px;
}

@media (max-width: 768px) {
  .header {
    flex-wrap: wrap;
    gap: 12px;
    height: auto;
    padding: 12px 16px;
  }
  .header-center {
    order: 3;
    width: 100%;
    justify-content: center;
  }
}
</style>