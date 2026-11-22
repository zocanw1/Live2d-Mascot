<script setup lang="ts">
interface Props {
  stat: {
    label: string;
    value: string | number;
    icon: string;
    color: string;
  };
}

const iconMap: Record<string, string> = {
  models: '<rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>',
  play: '<polygon points="5 3 19 12 5 21 5 3"/>',
  stop: '<rect x="6" y="6" width="12" height="12"/>',
  model: '<circle cx="12" cy="12" r="10"/>',
  fps: '<polyline points="23 4 23 10 17 10"/><path d="M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>',
};
</script>

<template>
  <div class="stat-card">
    <div class="stat-icon" :style="{ background: stat.color + '20', color: stat.color }" v-html="iconMap[stat.icon] || iconMap.models" />
    <div class="stat-content">
      <p class="stat-label">{{ stat.label }}</p>
      <p class="stat-value">{{ stat.value }}</p>
    </div>
  </div>
</template>

<style scoped>
.stat-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  transition: all var(--transition);
}

.stat-card:hover {
  border-color: var(--border-light);
  background: var(--bg-hover);
}

.stat-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  stroke-width: 2;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--fg-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 2px;
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--fg-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>