<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import { useModelStore } from '@/stores/model';
import { useOverlayStore } from '@/stores/overlay';
import { computed } from 'vue';
import NavItem from './NavItem.vue';

const router = useRouter();
const route = useRoute();
const modelStore = useModelStore();
const overlayStore = useOverlayStore();

const navItems = [
  { path: '/', label: 'Dashboard', icon: 'dashboard' },
  { path: '/models', label: 'Models', icon: 'models', badge: modelStore.modelCount },
  { path: '/preview', label: 'Preview', icon: 'preview' },
  { path: '/settings', label: 'Settings', icon: 'settings' },
];
</script>

<template>
  <aside class="sidebar" role="navigation" aria-label="Main navigation">
    <div class="sidebar-header">
      <div class="logo">
        <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
        </svg>
        <span>Live2D Mascot</span>
      </div>
      <div class="version">v1.1.0 (preview)</div>
    </div>

    <nav class="nav-list">
      <NavItem
        v-for="item in navItems"
        :key="item.path"
        :item="item"
        :is-active="route.path === item.path"
        @click="router.push(item.path)"
      />
    </nav>

    <div class="sidebar-footer">
      <div class="overlay-status" v-if="overlayStore.isRunning">
        <span class="status-dot running"></span>
        <span>Overlay Running</span>
        <span class="pid">PID: {{ overlayStore.status.pid }}</span>
      </div>
      <div class="overlay-status" v-else>
        <span class="status-dot stopped"></span>
        <span>Overlay Stopped</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid var(--border);
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--fg-primary);
  font-weight: 600;
  font-size: 14px;
}

.logo svg {
  color: var(--accent);
}

.version {
  font-size: 11px;
  color: var(--fg-muted);
  margin-top: 4px;
}

.nav-list {
  flex: 1;
  padding: 8px;
  overflow-y: auto;
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  background: var(--bg-tertiary);
}

.overlay-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--fg-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.running {
  background: var(--success);
  box-shadow: 0 0 8px var(--success);
}

.status-dot.stopped {
  background: var(--fg-muted);
}

.pid {
  margin-left: auto;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--fg-muted);
}
</style>