import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { OverlayStatus } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export const useOverlayStore = defineStore('overlay', () => {
  const status = ref<OverlayStatus>({
    isRunning: false,
  });
  const isStarting = ref(false);
  const isStopping = ref(false);
  const error = ref<string | null>(null);

  const isRunning = computed(() => status.value.isRunning);

  // Listen for events from Tauri backend
  listen('overlay-started', (event: any) => {
    status.value = {
      isRunning: true,
      pid: event.payload?.pid,
      uptime: Date.now(),
    };
  });

  listen('overlay-stopped', () => {
    status.value = { isRunning: false };
  });

  async function checkOverlayStatus() {
    try {
      const result = await invoke('check_overlay_process');
      status.value = result as OverlayStatus;
    } catch (e) {
      status.value = { isRunning: false };
    }
  }

  async function startOverlay(modelPath: string, config: any) {
    if (status.value.isRunning || isStarting.value) return;
    
    isStarting.value = true;
    error.value = null;

    try {
      await invoke('start_overlay', {
        modelPath,
        config,
      });

      status.value = {
        isRunning: true,
        uptime: Date.now(),
      };
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      status.value = { isRunning: false };
      throw e;
    } finally {
      isStarting.value = false;
    }
  }

  async function stopOverlay() {
    if (!status.value.isRunning || isStopping.value) return;
    
    isStopping.value = true;

    try {
      await invoke('stop_overlay');
      status.value = { isRunning: false };
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      isStopping.value = false;
    }
  }

  async function restartOverlay(modelPath: string, config: any) {
    await stopOverlay();
    await new Promise(r => setTimeout(r, 500));
    await startOverlay(modelPath, config);
  }

  return {
    status,
    isStarting,
    isStopping,
    error,
    isRunning,
    checkOverlayStatus,
    startOverlay,
    stopOverlay,
    restartOverlay,
  };
});
