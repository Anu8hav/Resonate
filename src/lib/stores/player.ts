import { writable, derived, get } from 'svelte/store';
import type { PlaybackState, Track } from './types';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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
  currentTrack: null,
  isPlaying: false,
  positionSeconds: 0,
  volume: 0.75,
  queue: [],
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
export async function togglePlay(): Promise<void> {
  const s = get(playbackState);
  try {
    if (s.isPlaying) {
      await invoke('pause_track');
      playbackState.update((state) => ({ ...state, isPlaying: false }));
    } else {
      await invoke('resume_track');
      playbackState.update((state) => ({ ...state, isPlaying: true }));
    }
  } catch (err) {
    console.warn('Failed to toggle play via Tauri (fallback to mock):', err);
    playbackState.update((state) => ({ ...state, isPlaying: !state.isPlaying }));
  }
}

export async function play(track: Track, newQueue?: Track[]): Promise<void> {
  if (track.source === 'local' && track.filePath) {
    try {
      const duration = await invoke<number>('play_track', { filePath: track.filePath });
      playbackState.update((s) => ({
        ...s,
        currentTrack: { ...track, durationSeconds: duration },
        isPlaying: true,
        positionSeconds: 0,
        queue: newQueue ?? s.queue
      }));
    } catch (err) {
      console.warn('Failed to play local track via Tauri (fallback to mock):', err);
      fallbackPlay(track, newQueue);
    }
  } else {
    // Non-local or missing path (e.g., future subsonic)
    fallbackPlay(track, newQueue);
  }
  logToHistory(track);
}

function fallbackPlay(track: Track, newQueue?: Track[]): void {
  playbackState.update((s) => ({
    ...s,
    currentTrack: track,
    isPlaying: true,
    positionSeconds: 0,
    queue: newQueue ?? s.queue
  }));
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

export async function setPosition(seconds: number): Promise<void> {
  try {
    await invoke('seek_track', { positionSeconds: seconds });
    playbackState.update((s) => ({ ...s, positionSeconds: seconds }));
  } catch (err) {
    console.warn('Seek failed (mock fallback):', err);
    playbackState.update((s) => ({ ...s, positionSeconds: seconds }));
  }
}

export async function setVolume(vol: number): Promise<void> {
  const clamped = Math.max(0, Math.min(1, vol));
  try {
    await invoke('set_volume', { volume: clamped });
    playbackState.update((s) => ({ ...s, volume: clamped }));
  } catch (err) {
    console.warn('Volume set failed (mock fallback):', err);
    playbackState.update((s) => ({ ...s, volume: clamped }));
  }
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

export async function stop(): Promise<void> {
  try {
    await invoke('stop_track');
  } catch (err) {
    console.warn('Failed to stop track via Tauri (fallback to mock):', err);
  }
  playbackState.update((s) => ({
    ...s,
    isPlaying: false,
    positionSeconds: 0,
    currentTrack: null
  }));
}

export function skipNext(): void {
  const s = get(playbackState);
  if (s.queue.length === 0) return;
  
  const currentIndex = s.queue.findIndex(t => t.id === s.currentTrack?.id);
  let nextIndex = currentIndex + 1;
  
  if (nextIndex >= s.queue.length) {
    if (s.repeat === 'all') {
      nextIndex = 0;
    } else {
      return; // Stop at end of queue
    }
  }
  
  play(s.queue[nextIndex]);
}

/**
 * Handles track-ended events with proper repeat mode branching.
 * - repeat 'one': replay the same track
 * - repeat 'all': advance to next, wrapping to start at end of queue
 * - repeat 'off': advance to next, stop at end of queue
 */
export function handleTrackEnd(): void {
  const state = get(playbackState);
  if (!state.currentTrack) return;

  console.log('[player] handleTrackEnd: repeat =', state.repeat, 'current =', state.currentTrack.title);

  if (state.repeat === 'one') {
    // Replay the same track
    play(state.currentTrack);
  } else if (state.repeat === 'all') {
    if (state.queue.length === 0) {
      play(state.currentTrack);
      return;
    }
    const currentIndex = state.queue.findIndex(t => t.id === state.currentTrack?.id);
    const nextIndex = currentIndex >= 0 ? (currentIndex + 1) % state.queue.length : 0;
    play(state.queue[nextIndex]);
  } else {
    // 'off' — advance to next, but stop if at end of queue
    if (state.queue.length === 0) {
      stop();
      return;
    }
    const currentIndex = state.queue.findIndex(t => t.id === state.currentTrack?.id);
    if (currentIndex >= 0 && currentIndex + 1 < state.queue.length) {
      play(state.queue[currentIndex + 1]);
    } else {
      // End of queue, repeat off — stop playback
      stop();
    }
  }
}

export function skipPrevious(): void {
  const s = get(playbackState);
  if (s.queue.length === 0) return;
  
  const currentIndex = s.queue.findIndex(t => t.id === s.currentTrack?.id);
  
  // Restart track if we're past 3 seconds, or if we're at the first track and repeat is off
  if (s.positionSeconds > 3.0 || (currentIndex <= 0 && s.repeat !== 'all')) {
    setPosition(0);
    return;
  }
  
  const prevIndex = currentIndex <= 0 ? s.queue.length - 1 : currentIndex - 1;
  play(s.queue[prevIndex]);
}

/** Format seconds to m:ss */
export function formatTime(totalSeconds: number): string {
  const mins = Math.floor(totalSeconds / 60);
  const secs = Math.floor(totalSeconds % 60);
  return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
}
