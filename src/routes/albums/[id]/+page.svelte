<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { currentTrack, play, formatTime } from '$lib/stores/player';
  import { Volume2, ArrowLeft, Play } from 'lucide-svelte';
  import { goto } from '$app/navigation';
  import type { Album, Track } from '$lib/stores/types';

  let album: Album | null = null;
  let tracks: Track[] = [];
  let loading = true;
  let error = '';

  $: isSingle = album?.totalTracks === null || album?.totalTracks === 1;
  $: badgeText = isSingle ? 'SINGLE' : (album && album.locallyOwnedCount < (album.totalTracks || 0) ? `${album.locallyOwnedCount}/${album.totalTracks} TRACKS` : null);

  onMount(async () => {
    try {
      const albumId = $page.params.id;
      const data = await invoke<{ album: any, tracks: any[] }>('get_album_with_tracks', { albumId });
      
      album = {
        ...data.album,
        coverUrl: data.album.coverUrl ? convertFileSrc(data.album.coverUrl) : null
      };
      
      tracks = data.tracks.map(t => ({
        ...t,
        coverUrl: t.coverUrl ? convertFileSrc(t.coverUrl) : null
      }));
    } catch (err: any) {
      console.error('Failed to load album:', err);
      error = err.toString();
    } finally {
      loading = false;
    }
  });

  function playAlbum() {
    if (tracks.length > 0) {
      play(tracks[0], tracks);
    }
  }
</script>

<div class="album-detail-page">
  <!-- Top Nav / Back Button -->
  <div class="top-nav">
    <button class="back-btn" on:click={() => goto('/albums')} title="Back to Albums">
      <ArrowLeft size={20} />
      <span>Back</span>
    </button>
  </div>

  {#if loading}
    <div class="loading-state">Loading album...</div>
  {:else if error}
    <div class="error-state">{error}</div>
  {:else if album}
    <!-- Album Header -->
    <header class="album-header">
      <div class="header-cover-wrapper">
        <div class="cover-container">
          {#if badgeText}
            <div class="album-badge">{badgeText}</div>
          {/if}
          {#if album.coverUrl}
            <img src={album.coverUrl} alt={album.title} class="header-cover" />
          {:else}
            <div class="header-placeholder">NO COVER</div>
          {/if}
        </div>
      </div>
      
      <div class="header-info">
        <div class="header-metadata">Album</div>
        <h1 class="header-title">{album.title}</h1>
        <h2 class="header-artist">{album.artist}</h2>
        
        <div class="header-actions">
          <button class="play-btn" on:click={playAlbum}>
            <Play size={20} fill="currentColor" />
            <span>Play Album</span>
          </button>
        </div>
      </div>
    </header>

    <!-- Tracks Table -->
    {#if tracks.length > 0}
      <div class="table-container">
        <table class="tracks-table">
          <thead>
            <tr>
              <th class="col-num">#</th>
              <th class="col-title">TITLE</th>
              <th class="col-time">TIME</th>
            </tr>
          </thead>
          <tbody>
            {#each tracks as track, i (track.id)}
              {@const isCurrent = $currentTrack?.id === track.id}
              <!-- svelte-ignore a11y-click-events-have-key-events -->
              <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
              <tr class:playing={isCurrent} on:click={() => play(track, tracks)}>
                <td class="col-num">
                  {#if isCurrent}
                    <span class="playing-icon"><Volume2 size={14} /></span>
                  {:else}
                    <span class="track-num">{i + 1}</span>
                  {/if}
                </td>
                <td class="col-title">
                  <span class="title-text" class:bold={isCurrent}>{track.title}</span>
                </td>
                <td class="col-time">{formatTime(track.durationSeconds)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        <span class="empty-text">No tracks found for this album.</span>
      </div>
    {/if}
  {/if}
</div>

<style>
  .album-detail-page {
    display: flex;
    flex-direction: column;
    gap: var(--gutter);
    max-width: var(--max-content-width);
    padding-bottom: 80px;
  }

  .top-nav {
    display: flex;
    align-items: center;
    margin-bottom: var(--space-2);
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: transparent;
    border: none;
    color: var(--color-on-surface-variant);
    font-family: var(--font-body-md-family);
    font-size: var(--font-body-md-size);
    cursor: pointer;
    padding: 8px;
    border-radius: var(--radius-default);
    transition: background-color var(--transition-fast), color var(--transition-fast);
  }

  .back-btn:hover {
    background-color: var(--color-surface-container);
    color: var(--color-on-surface);
  }

  /* ── Header ────────────────────────────────────────────────────── */

  .album-header {
    display: flex;
    gap: var(--space-6);
    align-items: flex-end;
    margin-bottom: var(--space-4);
  }

  .header-cover-wrapper {
    flex-shrink: 0;
  }

  .cover-container {
    position: relative;
    width: 200px;
    height: 200px;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background-color: var(--color-surface-container);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  }

  .header-cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .header-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    color: var(--color-outline);
    text-transform: uppercase;
  }

  .album-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    background-color: var(--color-surface-container-highest);
    color: var(--color-on-surface-variant);
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    padding: 4px 8px;
    border-radius: var(--radius-pill);
    box-shadow: 0 2px 4px rgba(0,0,0,0.5);
    z-index: 1;
    max-width: calc(100% - 16px);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .header-info {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  .header-metadata {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-on-surface-variant);
    text-transform: uppercase;
    margin-bottom: var(--space-2);
  }

  .header-title {
    font-family: var(--font-headline-lg-family);
    font-size: calc(var(--font-headline-lg-size) * 1.5);
    line-height: 1.1;
    font-weight: 700;
    color: var(--color-on-surface);
    margin: 0;
    margin-bottom: var(--space-2);
  }

  .header-artist {
    font-family: var(--font-headline-sm-family);
    font-size: var(--font-headline-sm-size);
    color: var(--color-on-surface-variant);
    margin: 0;
    margin-bottom: var(--space-4);
  }

  .header-actions {
    display: flex;
    gap: var(--space-3);
  }

  .play-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background-color: var(--color-primary);
    color: var(--color-on-primary);
    border: none;
    padding: 10px 20px;
    border-radius: var(--radius-pill);
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    font-weight: var(--font-nav-caps-weight);
    cursor: pointer;
    transition: transform var(--transition-fast), background-color var(--transition-fast);
  }

  .play-btn:hover {
    background-color: var(--color-primary-variant, #b69df8);
    transform: scale(1.05);
  }

  .play-btn:active {
    transform: scale(0.95);
  }

  /* ── Table ─────────────────────────────────────────────────────── */

  .table-container {
    overflow-x: auto;
  }

  .tracks-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  .tracks-table th {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-on-surface-variant);
    text-transform: uppercase;
    text-align: left;
    padding: var(--space-2) var(--margin-sm);
    border-bottom: 1px solid var(--color-outline-variant);
    user-select: none;
  }

  .tracks-table td {
    padding: 10px var(--margin-sm);
    border-bottom: 1px solid var(--color-surface-container);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tracks-table tr {
    transition: background-color var(--transition-fast);
    cursor: pointer;
  }

  .tracks-table tbody tr:hover {
    background-color: var(--color-surface-container);
  }

  .tracks-table tbody tr.playing {
    background-color: var(--color-surface-container-high);
  }

  /* Column widths */
  .col-num { width: 48px; text-align: center; }
  .col-title { width: auto; }
  .col-time { width: 72px; text-align: right; }

  .tracks-table th.col-num,
  .tracks-table td.col-num { text-align: center; }
  .tracks-table th.col-time,
  .tracks-table td.col-time { text-align: right; }

  /* Cell typography */
  .track-num {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    font-weight: var(--font-mono-data-weight);
    color: var(--color-on-surface-variant);
  }

  .playing-icon {
    color: var(--color-primary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .title-text {
    font-family: var(--font-body-md-family);
    font-size: var(--font-body-md-size);
    line-height: var(--font-body-md-line-height);
    color: var(--color-on-surface);
  }

  .title-text.bold {
    font-weight: 600;
    color: var(--color-primary);
  }

  td.col-time {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    font-weight: var(--font-mono-data-weight);
    color: var(--color-on-surface-variant);
  }

  /* ── State ─────────────────────────────────────────────────────── */

  .loading-state, .error-state, .empty-state {
    padding: var(--margin-lg) 0;
    text-align: center;
    font-family: var(--font-mono-data-family);
    font-size: var(--font-body-md-size);
    color: var(--color-outline);
  }

  .error-state {
    color: var(--color-error, #cf6679);
  }
</style>
