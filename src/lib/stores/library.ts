import { writable, derived } from 'svelte/store';
import type { Album, Track, Playlist } from './types';

/* ── Mock Albums ─────────────────────────────────────────────────────── */

export const MOCK_ALBUMS: Album[] = [
  { id: 'alb-1', title: 'Quantum Resonance', artist: 'Aetherial Echoes', coverUrl: null, trackCount: 12, source: 'local' },
  { id: 'alb-2', title: 'Harmonic Horizons', artist: 'Celestial Frequency', coverUrl: null, trackCount: 9, source: 'subsonic' },
  { id: 'alb-3', title: 'Analog Dreams', artist: 'Synthetica', coverUrl: null, trackCount: 14, source: 'local' },
  { id: 'alb-4', title: 'Midnight Reverie', artist: 'Vapor Waveform', coverUrl: null, trackCount: 10, source: 'local' },
  { id: 'alb-5', title: 'Starlight Overture', artist: 'Orchestra Nova', coverUrl: null, trackCount: 8, source: 'subsonic' },
  { id: 'alb-6', title: 'Neon Architecture', artist: 'Digital Mirage', coverUrl: null, trackCount: 11, source: 'local' },
  { id: 'alb-7', title: 'Solar Winds', artist: 'Helios Collective', coverUrl: null, trackCount: 7, source: 'subsonic' },
  { id: 'alb-8', title: 'Infinite Loop', artist: 'Recursion', coverUrl: null, trackCount: 16, source: 'local' },
  { id: 'alb-9', title: 'Deep Currents', artist: 'Oceanic Drift', coverUrl: null, trackCount: 9, source: 'local' },
  { id: 'alb-10', title: 'Electric Cathedral', artist: 'Arc Voltage', coverUrl: null, trackCount: 13, source: 'subsonic' },
  { id: 'alb-11', title: 'Void Transmission', artist: 'Unknown Artist', coverUrl: null, trackCount: 6, source: 'local' },
  { id: 'alb-12', title: 'Amber Frequencies', artist: 'Warm Signal', coverUrl: null, trackCount: 10, source: 'local' },
  { id: 'alb-13', title: 'Ghost Patterns', artist: 'Spectral Audio', coverUrl: null, trackCount: 8, source: 'subsonic' },
  { id: 'alb-14', title: 'Pulse Width', artist: 'Modular Synthesis', coverUrl: null, trackCount: 15, source: 'local' },
  { id: 'alb-15', title: 'Chromatic Shift', artist: 'Prism Effect', coverUrl: null, trackCount: 11, source: 'local' },
  { id: 'alb-16', title: 'Terminal Velocity', artist: 'Free Fall', coverUrl: null, trackCount: 9, source: 'subsonic' },
  { id: 'alb-17', title: 'Frozen Light', artist: 'Absolute Zero', coverUrl: null, trackCount: 7, source: 'local' },
  { id: 'alb-18', title: 'Magnetic North', artist: 'Polar Shift', coverUrl: null, trackCount: 12, source: 'local' },
];

/* ── Mock Tracks ─────────────────────────────────────────────────────── */

export const MOCK_TRACKS: Track[] = [
  { id: 'trk-1', title: 'Quantum Resonance', artist: 'Aetherial Echoes', album: 'Quantum Resonance', durationSeconds: 252, coverUrl: null, source: 'local', filePath: '/music/01.flac' },
  { id: 'trk-2', title: 'Neon Pulse', artist: 'Synthetica', album: 'Analog Dreams', durationSeconds: 312, coverUrl: null, source: 'local', filePath: '/music/02.flac' },
  { id: 'trk-3', title: 'Acoustic Drift', artist: 'Celestial Frequency', album: 'Harmonic Horizons', durationSeconds: 195, coverUrl: null, source: 'subsonic', streamUrl: 'https://sub.local/stream/3' },
  { id: 'trk-4', title: 'Midnight Rain', artist: 'Vapor Waveform', album: 'Midnight Reverie', durationSeconds: 284, coverUrl: null, source: 'local', filePath: '/music/04.flac' },
  { id: 'trk-5', title: 'Overture in C Minor', artist: 'Orchestra Nova', album: 'Starlight Overture', durationSeconds: 478, coverUrl: null, source: 'subsonic', streamUrl: 'https://sub.local/stream/5' },
  { id: 'trk-6', title: 'Glass Towers', artist: 'Digital Mirage', album: 'Neon Architecture', durationSeconds: 226, coverUrl: null, source: 'local', filePath: '/music/06.flac' },
  { id: 'trk-7', title: 'Coronal Mass', artist: 'Helios Collective', album: 'Solar Winds', durationSeconds: 341, coverUrl: null, source: 'subsonic', streamUrl: 'https://sub.local/stream/7' },
  { id: 'trk-8', title: 'Stack Overflow', artist: 'Recursion', album: 'Infinite Loop', durationSeconds: 198, coverUrl: null, source: 'local', filePath: '/music/08.flac' },
  { id: 'trk-9', title: 'Abyssal Plain', artist: 'Oceanic Drift', album: 'Deep Currents', durationSeconds: 267, coverUrl: null, source: 'local', filePath: '/music/09.flac' },
  { id: 'trk-10', title: 'High Voltage Hymn', artist: 'Arc Voltage', album: 'Electric Cathedral', durationSeconds: 303, coverUrl: null, source: 'subsonic', streamUrl: 'https://sub.local/stream/10' },
  { id: 'trk-11', title: 'Signal Lost', artist: 'Unknown Artist', album: 'Void Transmission', durationSeconds: 189, coverUrl: null, source: 'local', filePath: '/music/11.flac' },
  { id: 'trk-12', title: 'Warm Frequency', artist: 'Warm Signal', album: 'Amber Frequencies', durationSeconds: 245, coverUrl: null, source: 'local', filePath: '/music/12.flac' },
  { id: 'trk-13', title: 'Afterimage', artist: 'Spectral Audio', album: 'Ghost Patterns', durationSeconds: 356, coverUrl: null, source: 'subsonic', streamUrl: 'https://sub.local/stream/13' },
  { id: 'trk-14', title: 'Sawtooth Dreams', artist: 'Modular Synthesis', album: 'Pulse Width', durationSeconds: 271, coverUrl: null, source: 'local', filePath: '/music/14.flac' },
];

/* ── Mock Playlists ──────────────────────────────────────────────────── */

export const MOCK_PLAYLISTS: Playlist[] = [
  { id: 'pl-1', title: 'Favorites', coverUrl: null, trackIds: ['trk-1', 'trk-5', 'trk-7', 'trk-9', 'trk-13'] },
  { id: 'pl-2', title: 'Late Night', coverUrl: null, trackIds: ['trk-4', 'trk-6', 'trk-11', 'trk-12'] },
  { id: 'pl-3', title: 'High Energy', coverUrl: null, trackIds: ['trk-2', 'trk-8', 'trk-10', 'trk-14', 'trk-3', 'trk-7'] },
];

/* ── Writable Stores ─────────────────────────────────────────────────── */

export const albums = writable<Album[]>(MOCK_ALBUMS);
export const tracks = writable<Track[]>(MOCK_TRACKS);
export const playlists = writable<Playlist[]>(MOCK_PLAYLISTS);

/* ── Derived Stores ──────────────────────────────────────────────────── */

/** Unique artist entries derived from albums — no separate mock dataset needed */
export const artists = derived(albums, ($albums) => {
  const artistMap = new Map<string, { name: string; albumCount: number; imageUrl: string | null }>();
  for (const album of $albums) {
    const existing = artistMap.get(album.artist);
    if (existing) {
      existing.albumCount++;
    } else {
      artistMap.set(album.artist, { name: album.artist, albumCount: 1, imageUrl: null });
    }
  }
  return Array.from(artistMap.values()).sort((a, b) => a.name.localeCompare(b.name));
});
