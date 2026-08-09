<script lang="ts">
  import {
    currentTrack,
    isPlaying,
    positionSeconds,
    volume,
    togglePlay,
    setPosition,
    setVolume,
    formatTime,
    playbackState,
    toggleExpandedView
  } from '$lib/stores/player';
  import { SkipBack, SkipForward, Play, Pause, Volume2, VolumeX, Maximize2 } from 'lucide-svelte';

  function handleSeek(e: Event) {
    const target = e.target as HTMLInputElement;
    setPosition(parseFloat(target.value));
  }

  function handleVolume(e: Event) {
    const target = e.target as HTMLInputElement;
    setVolume(parseFloat(target.value));
  }
</script>

<footer class="now-playing-bar">
  <!-- Left: Track Info -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="track-info" on:click={toggleExpandedView}>
    {#if $currentTrack}
      <div class="track-thumbnail">
        {#if $currentTrack.coverUrl}
          <img src={$currentTrack.coverUrl} alt={$currentTrack.title} class="thumb-img" />
        {:else}
          <div class="thumb-placeholder">
            <span class="no-cover-label">NO COVER</span>
          </div>
        {/if}
      </div>
      <div class="track-meta">
        <span class="track-title">{$currentTrack.title}</span>
        <span class="track-artist">{$currentTrack.artist}</span>
      </div>
    {:else}
      <div class="track-meta">
        <span class="track-title empty">No track playing</span>
      </div>
    {/if}
  </div>

  <!-- Center: Transport Controls + Progress -->
  <div class="transport">
    <div class="transport-buttons">
      <button class="transport-btn" title="Previous">
        <SkipBack size={18} />
      </button>

      <button class="play-pause-btn" on:click={togglePlay} title={$isPlaying ? 'Pause' : 'Play'}>
        {#if $isPlaying}
          <Pause size={18} />
        {:else}
          <Play size={18} style="margin-left: 2px;" />
        {/if}
      </button>

      <button class="transport-btn" title="Next">
        <SkipForward size={18} />
      </button>
    </div>

    <div class="progress-row">
      <span class="time-label">{formatTime($positionSeconds)}</span>
      <input
        type="range"
        min="0"
        max={$currentTrack?.durationSeconds ?? 0}
        value={$positionSeconds}
        on:input={handleSeek}
        class="progress-slider"
      />
      <span class="time-label">{formatTime($currentTrack?.durationSeconds ?? 0)}</span>
    </div>
  </div>

  <!-- Right: Volume -->
  <div class="volume-section">
    <button class="transport-btn" title="Volume">
      {#if $volume === 0}
        <VolumeX size={18} />
      {:else}
        <Volume2 size={18} />
      {/if}
    </button>
    <input
      type="range"
      min="0"
      max="1"
      step="0.01"
      value={$volume}
      on:input={handleVolume}
      class="volume-slider"
    />
    <button class="transport-btn expand-btn" title="Expand Now Playing" on:click={toggleExpandedView}>
      <Maximize2 size={16} />
    </button>
  </div>
</footer>

<style>
  .now-playing-bar {
    height: var(--player-bar-height);
    background-color: var(--color-surface-container-low);
    border-top: 1px solid var(--color-outline-variant);
    display: flex;
    align-items: center;
    padding: 0 var(--gutter);
    gap: var(--gutter);
    user-select: none;
    flex-shrink: 0;
    z-index: 90;
  }

  /* ── Track Info (Left) ─────────────────────────────────────────── */

  .track-info {
    display: flex;
    align-items: center;
    gap: var(--margin-sm);
    width: 260px;
    min-width: 180px;
    flex-shrink: 0;
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }

  .track-info:hover {
    opacity: 0.8;
  }

  .track-thumbnail {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-default);
    overflow: hidden;
    flex-shrink: 0;
  }

  .thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .thumb-placeholder {
    width: 100%;
    height: 100%;
    background-color: var(--color-surface-container-high);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .no-cover-label {
    font-family: var(--font-mono-label-family);
    font-size: 6px;
    font-weight: var(--font-mono-label-weight);
    letter-spacing: var(--font-mono-label-letter-spacing);
    color: var(--color-outline);
    text-transform: uppercase;
  }

  .track-meta {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .track-title {
    font-family: var(--font-body-md-family);
    font-size: var(--font-body-md-size);
    line-height: var(--font-body-md-line-height);
    font-weight: 600;
    color: var(--color-on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .track-title.empty {
    color: var(--color-outline);
    font-weight: 400;
  }

  .track-artist {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    line-height: var(--font-body-sm-line-height);
    color: var(--color-on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ── Transport Controls (Center) ───────────────────────────────── */

  .transport {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    max-width: 560px;
    margin: 0 auto;
  }

  .transport-buttons {
    display: flex;
    align-items: center;
    gap: var(--margin-sm);
  }

  .transport-btn {
    color: var(--color-on-surface-variant);
    padding: 6px;
    border-radius: var(--radius-pill);
    transition: color var(--transition-fast);
  }

  .transport-btn:hover {
    color: var(--color-on-surface);
  }

  .play-pause-btn {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background-color: var(--color-primary-container);
    color: var(--color-on-primary-container);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color var(--transition-fast);
  }

  .play-pause-btn:hover {
    background-color: var(--color-primary);
    color: var(--color-on-primary);
  }

  .progress-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
  }

  .time-label {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    line-height: var(--font-mono-data-line-height);
    font-weight: var(--font-mono-data-weight);
    color: var(--color-on-surface-variant);
    min-width: 36px;
    text-align: center;
  }

  .progress-slider,
  .volume-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--color-surface-container-highest);
    outline: none;
    cursor: pointer;
  }

  .progress-slider::-webkit-slider-thumb,
  .volume-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-primary);
    cursor: pointer;
    border: none;
  }

  .progress-slider::-webkit-slider-thumb:hover,
  .volume-slider::-webkit-slider-thumb:hover {
    background: var(--color-primary-container);
    transform: scale(1.2);
  }

  /* ── Volume (Right) ────────────────────────────────────────────── */

  .volume-section {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 180px;
    min-width: 150px;
    flex-shrink: 0;
    justify-content: flex-end;
  }

  .expand-btn {
    margin-left: var(--space-1);
  }
</style>
