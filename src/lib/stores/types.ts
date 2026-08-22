export interface Track {
  id: string;
  title: string;
  artist: string;
  album: string | null;
  durationSeconds: number;
  coverUrl: string | null;
  source: 'local' | 'subsonic';
  filePath?: string;
  streamUrl?: string;
  format?: string;        // e.g. "FLAC", "MP3"
  bitrate?: number;       // in kbps
  sampleRate?: number;    // in Hz, e.g. 48000
  bitDepth?: number;      // e.g. 16, 24 (lossless only)
  channels?: number;      // e.g. 1 = mono, 2 = stereo
}

export interface Album {
  id: string;
  title: string;
  artist: string;
  coverUrl: string | null;
  totalTracks: number | null;
  locallyOwnedCount: number;
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
  dynamicAccentColor: boolean;
  cacheUsedGB: number;
  cacheTotalGB: number;
}

export interface Playlist {
  id: string;
  title: string;
  coverUrl: string | null;
  trackIds: string[];
}

export interface ScanSummary {
  tracksFound: number;
  tracksSkipped: number;
  albumsFound: number;
  artistsFound: number;
}

