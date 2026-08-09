/** Resonate — Shared TypeScript interfaces for stores and components */

export interface Track {
  id: string;
  title: string;
  artist: string;
  album: string;
  durationSeconds: number;
  coverUrl: string | null;
  source: 'local' | 'subsonic';
  filePath?: string;
  streamUrl?: string;
}

export interface Album {
  id: string;
  title: string;
  artist: string;
  coverUrl: string | null;
  trackCount: number;
  source: 'local' | 'subsonic';
}

export interface PlaybackState {
  currentTrack: Track | null;
  isPlaying: boolean;
  positionSeconds: number;
  volume: number;
  queue: Track[];
  isExpandedViewOpen: boolean;
  shuffle: boolean;
  repeat: 'off' | 'all' | 'one';
}

export interface ServerConfig {
  url: string;
  username: string;
  apiToken: string;
  apiVersion: string;
  connectionStatus: 'online' | 'offline' | 'checking';
  latencyMs: number;
  lastSyncAt: string;
}

export interface SettingsState {
  transcodeFLACToOpus: boolean;
  scrobbleToLastFm: boolean;
  cacheUsedGB: number;
  cacheTotalGB: number;
}

export interface Playlist {
  id: string;
  title: string;
  coverUrl: string | null;
  trackIds: string[];
}
