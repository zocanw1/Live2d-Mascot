export interface Live2DModel {
  id: string;
  name: string;
  path: string;
  thumbnail?: string;
  expressions: string[];
  motions: string[];
  textures: string[];
  addedAt: number;
  lastUsedAt?: number;
}

export interface OverlayConfig {
  modelId: string;
  position: { x: number; y: number };
  scale: number;
  lookAtEnabled: boolean;
  clickThroughLock: boolean;
  fpsLimit: 30 | 60;
  opacity: number;
  alwaysOnTop: boolean;
}

export interface AppSettings {
  theme: 'dark' | 'light' | 'system';
  language: 'en' | 'id';
  autoStartOverlay: boolean;
  minimizeToTray: boolean;
  closeToTray: boolean;
  overlayConfigPath: string;
  modelsDirectory: string;
}

export interface OverlayStatus {
  isRunning: boolean;
  pid?: number;
  uptime?: number;
  memoryUsage?: number;
  cpuUsage?: number;
}

export type ViewMode = 'dashboard' | 'models' | 'settings';