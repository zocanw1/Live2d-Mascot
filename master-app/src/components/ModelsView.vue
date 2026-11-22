<script setup lang="ts">
import { useModelStore } from '@/stores/model';
import { useOverlayStore } from '@/stores/overlay';
import { useSettingsStore } from '@/stores/settings';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import ConfirmDialog from './ConfirmDialog.vue';
import ModelCard from './ModelCard.vue';

const modelStore = useModelStore();
const overlayStore = useOverlayStore();
const settingsStore = useSettingsStore();

const searchQuery = ref('');
const showDeleteConfirm = ref(false);
const modelToDelete = ref<string | null>(null);

const filteredModels = computed(() => {
  if (!searchQuery.value) return modelStore.models;
  const q = searchQuery.value.toLowerCase();
  return modelStore.models.filter(m => 
    m.name.toLowerCase().includes(q) || m.id.toLowerCase().includes(q)
  );
});

async function handleDelete(id: string) {
  modelToDelete.value = id;
  showDeleteConfirm.value = true;
}

async function confirmDelete() {
  if (modelToDelete.value) {
    await modelStore.removeModel(modelToDelete.value);
    modelToDelete.value = null;
  }
  showDeleteConfirm.value = false;
}

async function setAsActive(id: string) {
  modelStore.selectModel(id);
  settingsStore.updateOverlayConfig('modelId', id);
}
</script>

<template>
  <div class="models-view">
    <div class="view-header">
      <div>
        <h1>Models</h1>
        <p class="subtitle">Manage your Live2D models</p>
      </div>
      <button class="btn btn-primary" @click="modelStore.importModel()">
        <span class="btn-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg></span>
        Import Model
      </button>
    </div>

    <div class="search-bar">
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search models..."
        v-model="searchQuery"
        aria-label="Search models"
      />
    </div>

    <div class="models-grid" v-if="filteredModels.length > 0">
      <ModelCard
        v-for="model in filteredModels"
        :key="model.id"
        :model="model"
        :is-active="modelStore.selectedModelId === model.id"
        :is-running="overlayStore.isRunning && overlayStore.status.pid && modelStore.selectedModelId === model.id"
        @select="setAsActive"
        @delete="handleDelete"
        @start="() => overlayStore.startOverlay(model.path, { ...settingsStore.overlayConfig, modelId: model.id, modelPath: model.path })"
      />
    </div>

    <div class="empty-state" v-else>
      <svg viewBox="0 0 24 24" width="64" height="64" fill="none" stroke="currentColor" stroke-width="1">
        <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
      </svg>
      <h3 v-if="searchQuery">No models found</h3>
      <h3 v-else>No models yet</h3>
      <p v-if="searchQuery">Try a different search term</p>
      <p v-else>Import your first Live2D model to get started</p>
      <button class="btn btn-primary" @click="modelStore.importModel()">
        <span class="btn-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg></span>
        Import Model
      </button>
    </div>

    <ConfirmDialog
      v-model:show="showDeleteConfirm"
      title="Delete Model?"
      :message="`Are you sure you want to remove '${modelStore.models.find(m => m.id === modelToDelete)?.name || modelToDelete}'? This only removes it from the list, the files remain on disk.`"
      confirm-text="Delete"
      variant="danger"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.models-view {
  max-width: 1000px;
  margin: 0 auto;
}

.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
  flex-wrap: wrap;
  gap: 16px;
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

.search-bar {
  position: relative;
  max-width: 400px;
  margin-bottom: 20px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 18px;
  height: 18px;
  color: var(--fg-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 44px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  color: var(--fg-primary);
  font-size: 13px;
  transition: all var(--transition);
}

.search-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-light);
}

.search-input::placeholder {
  color: var(--fg-muted);
}

.models-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
  background: var(--bg-secondary);
  border: 1px dashed var(--border);
  border-radius: var(--radius-lg);
}

.empty-state svg {
  color: var(--border-light);
  margin-bottom: 16px;
}

.empty-state h3 {
  font-size: 18px;
  font-weight: 600;
  color: var(--fg-primary);
  margin-bottom: 8px;
}

.empty-state p {
  font-size: 14px;
  color: var(--fg-secondary);
  margin-bottom: 20px;
}
</style>