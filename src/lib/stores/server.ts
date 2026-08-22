import { writable } from 'svelte/store';
import type { ServerConfig, SettingsState } from './types';

export const serverConfig = writable<ServerConfig>({
  url: 'https://music.example.com',
  username: 'anubhav',
  apiToken: '••••••••••••••••••••',
  apiVersion: '1.16.1',
  connectionStatus: 'online',
  latencyMs: 12,
  lastSyncAt: '2026-08-08 12:34:00'
});

export const settingsState = writable<SettingsState>({
  transcodeFLACToOpus: false,
  scrobbleToLastFm: true,
  dynamicAccentColor: true,
  cacheUsedGB: 4.2,
  cacheTotalGB: 10.0
});
