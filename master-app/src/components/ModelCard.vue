<script setup lang="ts">
interface Props {
  model: {
    id: string;
    name: string;
    path: string;
    thumbnail?: string;
    expressions: string[];
    motions: string[];
    textures: string[];
    addedAt: number;
    lastUsedAt?: number;
  };
  isActive: boolean;
  isRunning: boolean;
}

interface Emits {
  select: [string];
  delete: [string];
  start: [];
}

defineEmits<Emits>();

function formatDate(ts: number) {
  return new Date(ts).toLocaleDateString('id-ID', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  });
}
</script>

<template>
  <div class="model-card" :class="{ active: isActive, running: isRunning }">
    <div class="model-thumbnail">
      <img v-if="model.thumbnail" :src="model.thumbnail" :alt="model.name" />
      <div v-else class="thumbnail-placeholder">
        <svg viewBox="0 0 24 24" width="32" height="32" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
        </svg>
      </div>
      <div class="thumbnail-overlay" v-if="isRunning">
        <span class="running-badge">LIVE</span>
      </div>
    </div>

    <div class="model-content">
      <div class="model-header">
        <h3 class="model-name">{{ model.name }}</h3>
        <div class="model-actions">
          <button class="icon-btn" @click="$emit('select', model.id)" :aria-label="isActive ? 'Already active' : 'Set as active'" :disabled="isActive">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
          </button>
          <button class="icon-btn danger" @click="$emit('delete', model.id)" aria-label="Remove model">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
          </button>
        </div>
      </div>

      <p class="model-id">{{ model.id }}</p>

      <div class="model-meta">
        <span class="meta-item">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
          {{ model.expressions.length }} expr
        </span>
        <span class="meta-item">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5"/></svg>
          {{ model.motions.length }} mot
        </span>
        <span class="meta-item">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
          {{ model.textures.length }} tex
        </span>
      </div>

      <div class="model-footer">
        <span class="added-date">Added {{ formatDate(model.addedAt) }}</span>
        <button 
          class="btn btn-primary btn-sm" 
          @click="$emit('start')"
          :disabled="isRunning"
        >
          <span class="btn-icon"><svg viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg></span>
          {{ isRunning ? 'Running' : 'Start' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.model-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  transition: all var(--transition);
}

.model-card:hover {
  border-color: var(--border-light);
}

.model-card.active {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.model-card.running {
  border-color: var(--success);
}

.model-thumbnail {
  position: relative;
  aspect-ratio: 16/9;
  background: var(--bg-tertiary);
  overflow: hidden;
}

.model-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 300ms ease;
}

.model-card:hover .model-thumbnail img {
  transform: scale(1.05);
}

.thumbnail-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-muted);
}

.thumbnail-overlay {
  position: absolute;
  top: 8px;
  right: 8px;
}

.running-badge {
  background: var(--success);
  color: var(--bg-primary);
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.model-content {
  padding: 16px;
}

.model-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}

.model-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--fg-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.model-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-muted);
  transition: all var(--transition);
}

.icon-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.icon-btn.danger:hover:not(:disabled) {
  background: rgba(244, 71, 71, 0.15);
  color: var(--error);
}

.icon-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.model-id {
  font-size: 11px;
  color: var(--fg-muted);
  font-family: 'JetBrains Mono', monospace;
  margin-bottom: 12px;
}

.model-meta {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--fg-secondary);
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
}

.model-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.added-date {
  font-size: 11px;
  color: var(--fg-muted);
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}
</style>