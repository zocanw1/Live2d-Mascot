<script setup lang="ts">
import { useOverlayStore } from '@/stores/overlay';
import { useSettingsStore } from '@/stores/settings';
import { computed } from 'vue';

defineEmits(['hide']);

const overlayStore = useOverlayStore();
const settingsStore = useSettingsStore();

const uptime = computed(() => {
  if (!overlayStore.status.uptime) return '0s';
  const ms = Date.now() - overlayStore.status.uptime;
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  
  if (hours > 0) return `${hours}h ${minutes % 60}m`;
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
  return `${seconds}s`;
});

const memory = computed(() => {
  if (!overlayStore.status.memoryUsage) return '0 MB';
  const mb = (overlayStore.status.memoryUsage / 1024 / 1024).toFixed(1);
  return `${mb} MB`;
});
</script>

<template>
  <div class="status-bar" role="status" aria-live="polite">
    <div class="status-item">
      <span class="status-dot running"></span>
      <span>Overlay Active</span>
    </div>
    <div class="status-divider"></div>
    <div class="status-item">
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/></svg>
      <span>Uptime: {{ uptime }}</span>
    </div>
    <div class="status-divider"></div>
    <div class="status-item">
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="20" rx="2"/><path d="M6 10h12"/><path d="M6 14h12"/></svg>
      <span>RAM: {{ memory }}</span>
    </div>
    <div class="status-divider"></div>
    <div class="status-item">
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
      <span>{{ settingsStore.overlayConfig.fpsLimit }} FPS</span>
    </div>
    <div class="status-divider"></div>
    <button class="status-hide-btn" @click="$emit('hide')" aria-label="Hide status bar">
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    </button>
  </div>
</template>

<style scoped>
.status-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 8px 20px;
  background: var(--overlay-bg);
  border-top: 1px solid var(--border);
  backdrop-filter: blur(10px);
  z-index: 999;
  font-size: 12px;
  color: var(--fg-secondary);
  box-shadow: 0 -4px 20px var(--shadow);
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-divider {
  width: 1px;
  height: 16px;
  background: var(--border);
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

.status-hide-btn {
  width: 24px;
  height: 24px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-muted);
  margin-left: 8px;
}

.status-hide-btn:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

@media (max-width: 768px) {
  .status-bar {
    flex-wrap: wrap;
    padding: 6px 12px;
    gap: 8px;
  }
  .status-item span {
    display: none;
  }
  .status-divider {
    display: none;
  }
}
</style>