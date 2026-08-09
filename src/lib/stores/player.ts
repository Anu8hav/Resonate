import { writable, derived } from 'svelte/store';
import type { PlaybackState, Track } from './types';

const MOCK_TRACK: Track = {
  id: 'trk-1',
  title: 'Quantum Resonance',
  artist: 'Aetherial Echoes',
  album: 'Quantum Resonance',
  durationSeconds: 252,
  coverUrl: null,
  source: 'local',
  filePath: '/music/quantum-resonance/01-quantum-resonance.flac'
};

export const playbackState = writable<PlaybackState>({
  currentTrack: MOCK_TRACK,
  isPlaying: false,
  positionSeconds: 67,
  volume: 0.75,
  queue: [MOCK_TRACK, MOCK_TRACK, MOCK_TRACK], // Mock some queue items for testing
  isExpandedViewOpen: false,
  shuffle: false,
  repeat: 'off'
});

export const likedTrackIds = writable<Set<string>>(new Set(['trk-1']));
export const history = writable<Track[]>([]);

/* Derived convenience stores for component binding */
export const currentTrack = derived(playbackState, ($s) => $s.currentTrack);
export const isPlaying = derived(playbackState, ($s) => $s.isPlaying);
export const positionSeconds = derived(playbackState, ($s) => $s.positionSeconds);
export const volume = derived(playbackState, ($s) => $s.volume);
export const isExpandedViewOpen = derived(playbackState, ($s) => $s.isExpandedViewOpen);
export const shuffle = derived(playbackState, ($s) => $s.shuffle);
export const repeat = derived(playbackState, ($s) => $s.repeat);
export const queue = derived(playbackState, ($s) => $s.queue);

/* Actions */
export function togglePlay(): void {
  playbackState.update((s) => ({ ...s, isPlaying: !s.isPlaying }));
}

export function play(track: Track): void {
  playbackState.update((s) => ({
    ...s,
    currentTrack: track,
    isPlaying: true,
    positionSeconds: 0
  }));
  logToHistory(track);
}

function logToHistory(track: Track): void {
  history.update((h) => {
    // Avoid consecutive duplicates in history
    if (h.length > 0 && h[0].id === track.id) {
      return h;
    }
    return [track, ...h];
  });
}

export function setPosition(seconds: number): void {
  playbackState.update((s) => ({ ...s, positionSeconds: seconds }));
}

export function setVolume(vol: number): void {
  playbackState.update((s) => ({ ...s, volume: Math.max(0, Math.min(1, vol)) }));
}

export function toggleExpandedView(): void {
  playbackState.update((s) => ({ ...s, isExpandedViewOpen: !s.isExpandedViewOpen }));
}

export function toggleShuffle(): void {
  playbackState.update((s) => ({ ...s, shuffle: !s.shuffle }));
}

export function toggleRepeat(): void {
  playbackState.update((s) => {
    const next = s.repeat === 'off' ? 'all' : s.repeat === 'all' ? 'one' : 'off';
    return { ...s, repeat: next };
  });
}

export function toggleLike(trackId: string): void {
  likedTrackIds.update((set) => {
    const newSet = new Set(set);
    if (newSet.has(trackId)) {
      newSet.delete(trackId);
    } else {
      newSet.add(trackId);
    }
    return newSet;
  });
}

export function skipNext(): void {
  console.log('Mock skip next');
}

export function skipPrevious(): void {
  console.log('Mock skip previous');
}

/** Format seconds to m:ss */
export function formatTime(totalSeconds: number): string {
  const mins = Math.floor(totalSeconds / 60);
  const secs = Math.floor(totalSeconds % 60);
  return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
}
