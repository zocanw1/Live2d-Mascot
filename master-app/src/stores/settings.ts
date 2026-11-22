import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { OverlayConfig, AppSettings } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { BaseDirectory, readTextFile, writeTextFile, exists } from '@tauri-apps/plugin-fs';
import { join } from '@tauri-apps/api/path';

const DEFAULT_OVERLAY_CONFIG: OverlayConfig = {
  modelId: '',
  position: { x: 100, y: 100 },
  scale: 0.15,
  lookAtEnabled: true,
  clickThroughLock: false,
  fpsLimit: 60,
  opacity: 1.0,
  alwaysOnTop: true,
};

const DEFAULT_SETTINGS: AppSettings = {
  theme: 'dark',
  language: 'id',
  autoStartOverlay: false,
  minimizeToTray: true,
  closeToTray: true,
  overlayConfigPath: '',
  modelsDirectory: '',
};

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
  const overlayConfig = ref<OverlayConfig>({ ...DEFAULT_OVERLAY_CONFIG });
  const isLoading = ref(false);

  async function loadSettings() {
    isLoading.value = true;
    try {
      const configPath = await getConfigPath();
      if (await exists(configPath, { baseDir: BaseDirectory.AppData })) {
        const content = await readTextFile(configPath, { baseDir: BaseDirectory.AppData });
        const saved = JSON.parse(content);
        settings.value = { ...DEFAULT_SETTINGS, ...saved };
      }

      const overlayConfigPath = await getOverlayConfigPath();
      if (await exists(overlayConfigPath, { baseDir: BaseDirectory.AppData })) {
        const content = await readTextFile(overlayConfigPath, { baseDir: BaseDirectory.AppData });
        const saved = JSON.parse(content);
        overlayConfig.value = { ...DEFAULT_OVERLAY_CONFIG, ...saved };
      }
    } catch (e) {
      console.error('loadSettings error:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveSettings() {
    try {
      const configPath = await getConfigPath();
      await writeTextFile(configPath, JSON.stringify(settings.value, null, 2), { baseDir: BaseDirectory.AppData });
    } catch (e) {
      console.error('saveSettings error:', e);
    }
  }

  async function saveOverlayConfig() {
    try {
      const configPath = await getOverlayConfigPath();
      await writeTextFile(configPath, JSON.stringify(overlayConfig.value, null, 2), { baseDir: BaseDirectory.AppData });
      
      if (overlayConfig.value.modelId) {
        await invoke('notify_overlay_config_change', { config: overlayConfig.value });
      }
    } catch (e) {
      console.error('saveOverlayConfig error:', e);
    }
  }

  function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    settings.value[key] = value;
    saveSettings();
  }

  function updateOverlayConfig<K extends keyof OverlayConfig>(key: K, value: OverlayConfig[K]) {
    overlayConfig.value[key] = value;
    saveOverlayConfig();
  }

  function resetOverlayConfig() {
    overlayConfig.value = { ...DEFAULT_OVERLAY_CONFIG };
    saveOverlayConfig();
  }

  async function getConfigPath(): Promise<string> {
    try {
      const appData = await invoke('get_app_data_dir') as string;
      return join(appData, 'live2d-mascot', 'settings.json');
    } catch {
      return join('live2d-mascot', 'settings.json');
    }
  }

  async function getOverlayConfigPath(): Promise<string> {
    try {
      const appData = await invoke('get_app_data_dir') as string;
      return join(appData, 'live2d-mascot', 'overlay-config.json');
    } catch {
      return join('live2d-mascot', 'overlay-config.json');
    }
  }

  return {
    settings,
    overlayConfig,
    isLoading,
    loadSettings,
    saveSettings,
    saveOverlayConfig,
    updateSetting,
    updateOverlayConfig,
    resetOverlayConfig,
  };
});