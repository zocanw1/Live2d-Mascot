import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Live2DModel } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { BaseDirectory, readDir } from '@tauri-apps/plugin-fs';
import { join } from '@tauri-apps/api/path';

export const useModelStore = defineStore('model', () => {
  const models = ref<Live2DModel[]>([]);
  const selectedModelId = ref<string | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const selectedModel = computed(() => 
    models.value.find(m => m.id === selectedModelId.value) || models.value[0] || null
  );

  const modelCount = computed(() => models.value.length);

  async function loadModels() {
    isLoading.value = true;
    error.value = null;
    try {
      const modelsDir = await getModelsDirectory();
      try {
        const entries = await readDir(modelsDir, { baseDir: BaseDirectory.AppData, recursive: true });
        const modelFiles = entries.filter(e => e.name?.endsWith('.model3.json') || e.name === 'model3.json');
        
        for (const entry of modelFiles) {
          try {
            const modelPath = await join(modelsDir, entry.path || '');
            const modelDir = getDirName(modelPath);
            const model = await parseModel(modelPath, modelDir);
            if (model && !models.value.some(m => m.id === model.id)) {
              models.value.push(model);
            }
          } catch (e) {
            console.warn('Failed to load model:', entry.path, e);
          }
        }
      } catch (e) {
        // AppData models folder might be empty or not yet created
      }

      // Automatically add default IceGirl model if available
      const defaultIceGirl = 'C:\\Users\\ADVAN\\Downloads\\IceGirl_Live2d\\IceGIrl Live2D\\IceGirl.model3.json';
      if (!models.value.some(m => m.path === defaultIceGirl)) {
        try {
          const iceGirl = await addModel(defaultIceGirl);
          if (iceGirl && !selectedModelId.value) {
            selectedModelId.value = iceGirl.id;
          }
        } catch (e) {
          console.warn('Could not load default IceGirl model:', e);
        }
      }

      if (models.value.length > 0 && !selectedModelId.value) {
        selectedModelId.value = models.value[0].id;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load models';
      console.error('loadModels error:', e);
    } finally {
      isLoading.value = false;
    }
  }

  function getDirName(filePath: string): string {
    const norm = filePath.replace(/\\/g, '/');
    const lastSlash = norm.lastIndexOf('/');
    return lastSlash >= 0 ? norm.substring(0, lastSlash) : '';
  }

  async function parseModel(modelPath: string, modelDir: string): Promise<Live2DModel | null> {
    try {
      let content = '';
      try {
        content = await invoke('read_file_content', { path: modelPath });
      } catch {
        return null;
      }

      const modelJson = JSON.parse(content);
      const id = modelDir.split(/[/\\]/).filter(Boolean).pop() || 'IceGirl';
      const name = modelJson.Name || id;

      const expressions: string[] = [];
      const motions: string[] = [];
      const textures: string[] = modelJson.FileReferences?.Textures || [];

      if (modelJson.FileReferences?.Expressions) {
        for (const exp of modelJson.FileReferences.Expressions) {
          expressions.push(exp.Name || exp.File?.replace('.exp3.json', '') || 'unknown');
        }
      }

      if (modelJson.FileReferences?.Motions) {
        for (const [group, motionsList] of Object.entries(modelJson.FileReferences.Motions)) {
          if (Array.isArray(motionsList)) {
            for (const m of motionsList) {
              motions.push(`${group}:${m.File?.replace('.motion3.json', '') || 'unknown'}`);
            }
          }
        }
      }

      let thumbnail: string | undefined;
      try {
        if (textures[0]) {
          const texturePath = `${modelDir}/${textures[0]}`.replace(/\//g, '\\');
          thumbnail = await invoke('read_file_base64', { path: texturePath });
        }
      } catch {
        // Thumbnail load failure is non-fatal
      }

      return {
        id,
        name,
        path: modelPath,
        thumbnail,
        expressions,
        motions,
        textures,
        addedAt: Date.now(),
      };
    } catch (e) {
      console.error('parseModel error:', e);
      return null;
    }
  }

  async function addModel(filePath: string) {
    try {
      const modelDir = getDirName(filePath);
      const model = await parseModel(filePath, modelDir);
      if (model) {
        const existing = models.value.findIndex(m => m.id === model.id);
        if (existing >= 0) {
          models.value[existing] = model;
        } else {
          models.value.push(model);
        }
        selectedModelId.value = model.id;
        return model;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add model';
      console.error('addModel error:', e);
    }
    return null;
  }

  async function importModel() {
    // Import berbasis folder: pilih folder model (yang berisi .model3.json),
    // lalu salin seluruh folder ke folder models milik aplikasi.
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Live2D model folder (the folder containing the .model3.json)',
    });

    if (!selected) return null;

    const sourceDir = Array.isArray(selected) ? selected[0] : selected;
    try {
      const newModelPath = await invoke<string>('import_model_folder', { sourceDir });
      return await addModel(newModelPath);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error('importModel error:', e);
      return null;
    }
  }

  async function removeModel(id: string) {
    const idx = models.value.findIndex(m => m.id === id);
    if (idx >= 0) {
      models.value.splice(idx, 1);
      if (selectedModelId.value === id) {
        selectedModelId.value = models.value[0]?.id || null;
      }
    }
  }

  function selectModel(id: string) {
    selectedModelId.value = id;
    const model = models.value.find(m => m.id === id);
    if (model) {
      model.lastUsedAt = Date.now();
    }
  }

  async function getModelsDirectory(): Promise<string> {
    try {
      // get_app_data_dir mengembalikan <AppData>/live2d-mascot,
      // jadi cukup tambahkan "models" (tanpa menggandakan "live2d-mascot").
      const fullPath = await invoke('get_app_data_dir') as string;
      return join(fullPath, 'models');
    } catch {
      return join('live2d-mascot', 'models');
    }
  }

  return {
    models,
    selectedModelId,
    selectedModel,
    modelCount,
    isLoading,
    error,
    loadModels,
    addModel,
    importModel,
    removeModel,
    selectModel,
  };
});
