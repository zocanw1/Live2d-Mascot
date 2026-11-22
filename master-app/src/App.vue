<script setup lang="ts">
import { useOverlayStore } from '@/stores/overlay';
import { useModelStore } from '@/stores/model';
import { useSettingsStore } from '@/stores/settings';
import { onMounted, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import Sidebar from '@/components/Sidebar.vue';
import Header from '@/components/Header.vue';
import DashboardView from '@/components/DashboardView.vue';
import ModelsView from '@/components/ModelsView.vue';
import PreviewView from '@/components/PreviewView.vue';
import SettingsView from '@/components/SettingsView.vue';
import OverlayStatusBar from '@/components/OverlayStatusBar.vue';

const overlayStore = useOverlayStore();
const modelStore = useModelStore();
const settingsStore = useSettingsStore();
const router = useRouter();
const route = useRoute();

onMounted(async () => {
  await Promise.all([
    modelStore.loadModels(),
    settingsStore.loadSettings(),
    overlayStore.checkOverlayStatus(),
  ]);
});

const currentView = computed(() => {
  switch (route.path) {
    case '/models': return ModelsView;
    case '/preview': return PreviewView;
    case '/settings': return SettingsView;
    default: return DashboardView;
  }
});
</script>

<template>
  <div class="app">
    <Header />
    <div class="main-container">
      <Sidebar />
      <main class="content" role="main">
        <component :is="currentView" />
      </main>
    </div>
    <OverlayStatusBar v-if="overlayStore.isRunning" />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--bg-primary);
}

.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.content {
  flex: 1;
  overflow: auto;
  padding: 20px;
  background: var(--bg-primary);
}

@media (max-width: 1024px) {
  .content {
    padding: 16px;
  }
}
</style>