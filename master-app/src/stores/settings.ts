import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { OverlayConfig, AppSettings, VoiceConfig } from '@/types';
import { invoke } from '@tauri-apps/api/core';

const DEFAULT_VOICE_CONFIG: VoiceConfig = {
  enabled: false,
  groqApiKey: '',
  whisperModel: 'whisper-large-v3',
  llmEndpoint: 'http://localhost:20128/v1',
  llmApiKey: '',
  llmModel: 'claude-mycombo',
  systemPrompt: 'Kamu adalah asisten AI yang ramah dan membantu. Jawab singkat dan jelas dalam bahasa Indonesia.',
  ttsVoice: 'id-ID-GadisNeural',
  ttsEnabled: true,
  pttHotkey: 'Ctrl+Shift+Space',
  pttMouseButton: '',
};

const DEFAULT_OVERLAY_CONFIG: OverlayConfig = {
  modelId: '',
  position: { x: 100, y: 100 },
  scale: 0.15,
  lookAtEnabled: true,
  clickThroughLock: false,
  fpsLimit: 60,
  opacity: 1.0,
  alwaysOnTop: true,
  voice: DEFAULT_VOICE_CONFIG,
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
      const settingsContent = await invoke('load_config_file', { filename: 'settings.json' }) as string;
      if (settingsContent) {
        const saved = JSON.parse(settingsContent);
        settings.value = { ...DEFAULT_SETTINGS, ...saved };
      }

      const overlayContent = await invoke('load_config_file', { filename: 'overlay-config.json' }) as string;
      if (overlayContent) {
        const saved = JSON.parse(overlayContent);
        overlayConfig.value = {
          ...DEFAULT_OVERLAY_CONFIG,
          ...saved,
          voice: { ...DEFAULT_VOICE_CONFIG, ...(saved.voice || {}) },
        };
      }
    } catch (e) {
      console.error('loadSettings error:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function saveSettings() {
    try {
      await invoke('save_config_file', {
        filename: 'settings.json',
        content: JSON.stringify(settings.value, null, 2),
      });
    } catch (e) {
      console.error('saveSettings error:', e);
    }
  }

  async function saveOverlayConfig() {
    try {
      await invoke('save_config_file', {
        filename: 'overlay-config.json',
        content: JSON.stringify(overlayConfig.value, null, 2),
      });

      // Selalu push runtime config (live-update lookAtEnabled/opacity di overlay).
      await invoke('notify_overlay_config_change', { config: overlayConfig.value });
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