<script setup lang="ts">
import { useModelStore } from '@/stores/model';
import { useOverlayStore } from '@/stores/overlay';
import { useSettingsStore } from '@/stores/settings';
import { computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import StatCard from '@/components/StatCard.vue';
import QuickAction from '@/components/QuickAction.vue';

const modelStore = useModelStore();
const overlayStore = useOverlayStore();
const settingsStore = useSettingsStore();

const stats = computed(() => [
  { label: 'Models', value: modelStore.modelCount, icon: 'models', color: 'var(--accent)' },
  { label: 'Overlay', value: overlayStore.isRunning ? 'Running' : 'Stopped', icon: overlayStore.isRunning ? 'play' : 'stop', color: overlayStore.isRunning ? 'var(--success)' : 'var(--fg-muted)' },
  { label: 'Current Model', value: modelStore.selectedModel?.name || 'None', icon: 'model', color: 'var(--warning)' },
  { label: 'FPS Limit', value: `${settingsStore.overlayConfig.fpsLimit} FPS`, icon: 'fps', color: 'var(--fg-secondary)' },
]);

const actions = [
  { label: 'Import Model', icon: 'import', action: () => modelStore.importModel(), disabled: false },
  { label: 'Open Models Folder', icon: 'folder', action: openModelsFolder, disabled: modelStore.modelCount === 0 },
  { label: 'Refresh Models', icon: 'refresh', action: () => modelStore.loadModels(), disabled: modelStore.isLoading },
];

async function openModelsFolder() {
  try {
    await invoke('open_models_folder');
  } catch (e) {
    console.error('Open folder failed:', e);
  }
}

async function startOverlay() {
  // Handled by Header component
}
</script>

<template>
  <div class="dashboard">
    <section class="stats-section" aria-label="Statistics">
      <StatCard v-for="stat in stats" :key="stat.label" :stat="stat" />
    </section>

    <section class="actions-section" aria-label="Quick Actions">
      <h2 class="section-title">Quick Actions</h2>
      <div class="actions-grid">
        <QuickAction v-for="action in actions" :key="action.label" :action="action" />
      </div>
    </section>

    <section class="current-model-section" v-if="modelStore.selectedModel" aria-label="Current Model">
      <h2 class="section-title">Current Model</h2>
      <div class="model-card">
        <div class="model-preview">
          <img v-if="modelStore.selectedModel.thumbnail" :src="modelStore.selectedModel.thumbnail" :alt="modelStore.selectedModel.name" />
          <div v-else class="preview-placeholder">
            <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
            </svg>
          </div>
        </div>
        <div class="model-info">
          <h3>{{ modelStore.selectedModel.name }}</h3>
          <p class="model-id">{{ modelStore.selectedModel.id }}</p>
          <div class="model-meta">
            <span>{{ modelStore.selectedModel.expressions.length }} expressions</span>
            <span>{{ modelStore.selectedModel.motions.length }} motions</span>
            <span>{{ modelStore.selectedModel.textures.length }} textures</span>
          </div>
        </div>
        <div class="model-actions">
          <button class="btn btn-secondary" @click="$router.push('/models')">Manage</button>
          <button class="btn btn-primary" @click="startOverlay" :disabled="overlayStore.isRunning || overlayStore.isStarting">
            {{ overlayStore.isRunning ? 'Restart Overlay' : 'Start Overlay' }}
          </button>
        </div>
      </div>
    </section>

    <section class="empty-state" v-else aria-label="No model selected">
      <div class="empty-content">
        <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
        </svg>
        <h3>No Model Selected</h3>
        <p>Import a Live2D model (.model3.json) to get started</p>
        <button class="btn btn-primary" @click="modelStore.importModel()">
          <span class="btn-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg></span>
          Import Model
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.dashboard {
  max-width: 900px;
  margin: 0 auto;
}

.stats-section {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.actions-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-secondary);
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.actions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.current-model-section {
  margin-bottom: 24px;
}

.model-card {
  display: flex;
  gap: 16px;
  padding: 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}

.model-preview {
  width: 80px;
  height: 80px;
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--bg-tertiary);
  flex-shrink: 0;
}

.model-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 300ms ease;
}

.model-card:hover .model-preview img {
  transform: scale(1.05);
}

.preview-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-muted);
}

.model-info {
  flex: 1;
  min-width: 0;
}

.model-info h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--fg-primary);
  margin-bottom: 4px;
}

.model-id {
  font-size: 12px;
  color: var(--fg-muted);
  font-family: 'JetBrains Mono', monospace;
  margin-bottom: 8px;
}

.model-meta {
  display: flex;
  gap: 12px;
  font-size: 11px;
  color: var(--fg-secondary);
}

.model-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-shrink: 0;
}

.empty-state {
  padding: 60px 20px;
  text-align: center;
}

.empty-content {
  max-width: 320px;
  margin: 0 auto;
}

.empty-content svg {
  color: var(--border-light);
  margin-bottom: 16px;
}

.empty-content h3 {
  font-size: 18px;
  font-weight: 600;
  color: var(--fg-primary);
  margin-bottom: 8px;
}

.empty-content p {
  font-size: 14px;
  color: var(--fg-secondary);
  margin-bottom: 20px.
}

@media (max-width: 640px) {
  .model-card {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  
  .model-actions {
    flex-direction: row;
    width: 100%;
  }
  
  .model-actions .btn {
    flex: 1;
  }
}
</style>