<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { fade } from 'svelte/transition';
  import {
    currentTrack,
    isPlaying,
    positionSeconds,
    isExpandedViewOpen,
    shuffle,
    repeat,
    likedTrackIds,
    queue,
    togglePlay,
    toggleExpandedView,
    toggleShuffle,
    toggleRepeat,
    toggleLike,
    setPosition,
    formatTime,
    skipNext,
    skipPrevious
  } from '$lib/stores/player';
  import { playlists } from '$lib/stores/library';
  import {
    ChevronDown,
    Shuffle,
    SkipBack,
    Play,
    Pause,
    SkipForward,
    Repeat,
    Repeat1,
    Heart,
    ListPlus,
    Share,
    BarChart2,
    X
  } from 'lucide-svelte';

  // State
  let activeTab: 'upnext' | 'lyrics' = 'upnext';
  let ambientColor = 'var(--color-primary-container)';
  let isPlaylistPopoverOpen = false;
  let canvasEl: HTMLCanvasElement;

  // Watch for track changes to extract color
  $: if ($currentTrack) {
    extractColor($currentTrack.coverUrl);
  }

  // Handle click outside for playlist popover
  function handleWindowClick(e: MouseEvent) {
    if (isPlaylistPopoverOpen) {
      const target = e.target as HTMLElement;
      if (!target.closest('.playlist-action')) {
        isPlaylistPopoverOpen = false;
      }
    }
  }

  async function extractColor(src: string | null) {
    if (!src) {
      ambientColor = 'var(--color-primary-container)';
      return;
    }

    try {
      const img = new Image();
      img.crossOrigin = 'Anonymous';
      img.src = src;
      
      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });

      if (!canvasEl) return;
      const ctx = canvasEl.getContext('2d');
      if (!ctx) return;

      canvasEl.width = img.width;
      canvasEl.height = img.height;
      ctx.drawImage(img, 0, 0);

      // Simple average color sampling (stride by 40 pixels for speed)
      const imageData = ctx.getImageData(0, 0, canvasEl.width, canvasEl.height).data;
      let r = 0, g = 0, b = 0, count = 0;
      
      for (let i = 0; i < imageData.length; i += 4 * 40) {
        r += imageData[i];
        g += imageData[i + 1];
        b += imageData[i + 2];
        count++;
      }
      
      if (count > 0) {
        ambientColor = `rgb(${Math.round(r / count)}, ${Math.round(g / count)}, ${Math.round(b / count)})`;
      }
    } catch (e) {
      console.error('Failed to extract color', e);
      ambientColor = 'var(--color-primary-container)';
    }
  }

  function handleSeek(e: Event) {
    const target = e.target as HTMLInputElement;
    setPosition(parseFloat(target.value));
  }

  function formatTimeRemaining(pos: number, total: number) {
    const remaining = total - pos;
    return `-${formatTime(Math.max(0, remaining))}`;
  }

  onMount(() => {
    window.addEventListener('click', handleWindowClick);
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('click', handleWindowClick);
    }
  });
</script>

{#if $isExpandedViewOpen}
  <div class="expanded-overlay" transition:fade={{ duration: 200 }} style="--ambient-color: {ambientColor};">
    <!-- Hidden canvas for color extraction -->
    <canvas bind:this={canvasEl} style="display: none;"></canvas>

    <!-- Header -->
    <header class="overlay-header">
      <button class="close-btn" on:click={toggleExpandedView} title="Collapse">
        <ChevronDown size={24} />
      </button>
    </header>

    <div class="overlay-content">
      <!-- Ambient Wash Background -->
      <div class="ambient-wash"></div>

      <!-- Left Column: Art & Controls -->
      <div class="left-column">
        <div class="art-container">
          {#if $currentTrack?.coverUrl}
            <img src={$currentTrack.coverUrl} alt="Cover Art" class="cover-art" />
          {:else}
            <div class="cover-placeholder">
              <span>NO COVER</span>
            </div>
          {/if}
        </div>

        <div class="transport-controls">
          <button class="control-btn {$shuffle ? 'active' : ''}" on:click={toggleShuffle} title="Shuffle">
            <Shuffle size={20} />
          </button>
          
          <button class="control-btn" on:click={skipPrevious} title="Previous">
            <SkipBack size={24} />
          </button>
          
          <div class="play-wrapper">
            <div class="play-glow"></div>
            <button class="play-btn" on:click={togglePlay} title={$isPlaying ? 'Pause' : 'Play'}>
              {#if $isPlaying}
                <Pause size={24} />
              {:else}
                <Play size={24} style="margin-left: 2px;" />
              {/if}
            </button>
          </div>
          
          <button class="control-btn" on:click={skipNext} title="Next">
            <SkipForward size={24} />
          </button>
          
          <button class="control-btn {$repeat !== 'off' ? 'active' : ''}" on:click={toggleRepeat} title="Repeat">
            {#if $repeat === 'one'}
              <Repeat1 size={20} />
            {:else}
              <Repeat size={20} />
            {/if}
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
          <span class="time-label">{formatTimeRemaining($positionSeconds, $currentTrack?.durationSeconds ?? 0)}</span>
        </div>
      </div>

      <!-- Right Column: Info & Queue -->
      <div class="right-column">
        <!-- Metadata -->
        <div class="metadata-section">
          <h1 class="track-title">{$currentTrack?.title ?? 'No track playing'}</h1>
          <p class="track-artist-album">
            {$currentTrack?.artist ?? 'Unknown Artist'} &bull; {$currentTrack?.album ?? 'Unknown Album'}
          </p>
        </div>

        <!-- Actions -->
        <div class="action-row">
          <button 
            class="action-btn {$currentTrack && $likedTrackIds.has($currentTrack.id) ? 'active' : ''}" 
            on:click={() => $currentTrack && toggleLike($currentTrack.id)}
            title="Like"
          >
            <Heart size={20} class={$currentTrack && $likedTrackIds.has($currentTrack.id) ? 'fill-current' : ''} />
          </button>

          <div class="playlist-action">
            <button 
              class="action-btn" 
              on:click|stopPropagation={() => (isPlaylistPopoverOpen = !isPlaylistPopoverOpen)}
              title="Add to Playlist"
            >
              <ListPlus size={20} />
            </button>
            
            {#if isPlaylistPopoverOpen}
              <div class="playlist-popover" transition:fade={{ duration: 100 }}>
                <div class="popover-header">Add to Playlist</div>
                {#if $playlists.length > 0}
                  <div class="popover-list">
                    {#each $playlists as playlist}
                      <button class="popover-item" on:click={() => {
                        console.log('Added to', playlist.title);
                        isPlaylistPopoverOpen = false;
                      }}>
                        {playlist.title}
                      </button>
                    {/each}
                  </div>
                {:else}
                  <div class="popover-empty">No playlists</div>
                {/if}
              </div>
            {/if}
          </div>

          <button class="action-btn" title="Share">
            <Share size={20} />
          </button>
        </div>

        <!-- Badges -->
        <div class="badge-row">
          <span class="badge filled">FLAC 24-BIT / 48KHZ</span>
          <span class="badge outline">1411 KBPS</span>
          <span class="badge outline">STEREO</span>
        </div>

        <!-- Tabs -->
        <div class="tabs-section">
          <div class="tabs-header">
            <button 
              class="tab-btn {activeTab === 'upnext' ? 'active' : ''}"
              on:click={() => (activeTab = 'upnext')}
            >
              UP NEXT
            </button>
            <button 
              class="tab-btn {activeTab === 'lyrics' ? 'active' : ''}"
              on:click={() => (activeTab = 'lyrics')}
            >
              LYRICS
            </button>
          </div>

          <div class="tab-content">
            {#if activeTab === 'upnext'}
              <div class="queue-list">
                <!-- Current Track (Highlighted) -->
                {#if $currentTrack}
                  <button class="queue-row active-row">
                    <div class="queue-index">
                      <BarChart2 size={16} class="playing-icon" />
                    </div>
                    <div class="queue-meta">
                      <span class="queue-title">{$currentTrack.title}</span>
                      <span class="queue-artist">{$currentTrack.artist}</span>
                    </div>
                    <div class="queue-duration">{formatTime($currentTrack.durationSeconds)}</div>
                  </button>
                {/if}
                
                <!-- Queue Tracks -->
                {#each $queue as track, i}
                  <button class="queue-row">
                    <div class="queue-index">{i + 1}</div>
                    <div class="queue-meta">
                      <span class="queue-title">{track.title}</span>
                      <span class="queue-artist">{track.artist}</span>
                    </div>
                    <div class="queue-duration">{formatTime(track.durationSeconds)}</div>
                  </button>
                {/each}
                {#if $queue.length === 0}
                   <div class="queue-empty">Queue is empty</div>
                {/if}
              </div>
            {:else}
              <div class="lyrics-empty">
                <span class="empty-text">No lyrics available</span>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .expanded-overlay {
    position: fixed;
    top: 0;
    left: var(--sidebar-width);
    right: 0;
    bottom: 0;
    background-color: var(--color-background);
    z-index: 100;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Header ────────────────────────────────────────────────────── */

  .overlay-header {
    padding: var(--gutter);
    display: flex;
    justify-content: flex-start;
    z-index: 2;
  }

  .close-btn {
    color: var(--color-on-surface-variant);
    padding: 8px;
    border-radius: 50%;
    transition: all var(--transition-fast);
  }

  .close-btn:hover {
    color: var(--color-on-surface);
    background-color: var(--color-surface-container);
  }

  /* ── Layout & Ambient Wash ─────────────────────────────────────── */

  .overlay-content {
    flex: 1;
    display: flex;
    gap: 80px;
    padding: 0 80px 80px 80px;
    position: relative;
    max-width: 1600px;
    margin: 0 auto;
    width: 100%;
  }

  .ambient-wash {
    position: absolute;
    top: -10%;
    left: -10%;
    width: 60%;
    height: 120%;
    background-color: var(--ambient-color);
    filter: blur(120px);
    opacity: 0.2;
    z-index: 0;
    pointer-events: none;
    transition: background-color 0.6s ease-in-out;
    border-radius: 50%;
  }

  .left-column {
    flex: 0 0 500px;
    display: flex;
    flex-direction: column;
    gap: var(--margin-lg);
    z-index: 1;
  }

  .right-column {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    gap: var(--margin-lg);
    z-index: 1;
    min-width: 0;
    max-width: 100%;
    overflow-x: hidden;
  }

  /* ── Left Column: Art & Controls ───────────────────────────────── */

  .art-container {
    width: 500px;
    height: 500px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-outline-variant);
    overflow: hidden;
    background-color: var(--color-surface-container);
    box-shadow: 0 20px 40px rgba(0,0,0,0.4);
  }

  .cover-art {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-mono-label-family);
    font-size: 24px;
    letter-spacing: var(--font-mono-label-letter-spacing);
    color: var(--color-outline);
  }

  .transport-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--margin-sm);
  }

  .control-btn {
    color: var(--color-on-surface-variant);
    transition: color var(--transition-fast);
  }

  .control-btn:hover {
    color: var(--color-on-surface);
  }

  .control-btn.active {
    color: var(--color-primary);
  }

  .play-wrapper {
    position: relative;
    width: 64px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .play-glow {
    position: absolute;
    inset: -12px;
    background-color: var(--ambient-color);
    border-radius: 50%;
    filter: blur(16px);
    opacity: 0.6;
    transition: background-color 0.6s ease-in-out;
  }

  .play-btn {
    position: relative;
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background-color: var(--color-primary-container);
    color: var(--color-on-primary-container);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
    transition: transform var(--transition-fast), background-color var(--transition-fast);
  }

  .play-btn:hover {
    transform: scale(1.05);
    background-color: var(--color-primary);
    color: var(--color-on-primary);
  }

  .progress-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .time-label {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    color: var(--color-on-surface-variant);
    min-width: 48px;
  }

  .time-label:last-child {
    text-align: right;
  }

  .progress-slider {
    -webkit-appearance: none;
    appearance: none;
    flex: 1;
    height: 4px;
    border-radius: 2px;
    background: var(--color-surface-container-highest);
    outline: none;
    cursor: pointer;
  }

  .progress-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-primary);
    cursor: pointer;
    border: none;
  }

  .progress-slider::-webkit-slider-thumb:hover {
    background: var(--color-primary-container);
    transform: scale(1.2);
  }

  /* ── Right Column ──────────────────────────────────────────────── */

  .metadata-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .track-title {
    font-family: var(--font-headline-lg-family);
    font-size: 40px; /* Slightly larger for this view */
    line-height: 1.1;
    letter-spacing: var(--font-headline-lg-letter-spacing);
    font-weight: 700;
    color: var(--color-on-surface);
    overflow-wrap: break-word;
    word-break: break-word;
    white-space: normal;
  }

  .track-artist-album {
    font-family: var(--font-body-md-family);
    font-size: 18px;
    color: var(--color-on-surface-variant);
  }

  .action-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .action-btn {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-lg);
    background-color: var(--color-surface-container);
    color: var(--color-on-surface-variant);
    border: 1px solid var(--color-outline-variant);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .action-btn:hover {
    color: var(--color-on-surface);
    border-color: var(--color-outline);
  }

  .action-btn.active {
    color: var(--color-primary);
    border-color: var(--color-primary);
  }

  .action-btn :global(.fill-current) {
    fill: currentColor;
  }

  /* Playlist Popover */
  .playlist-action {
    position: relative;
  }

  .playlist-popover {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 8px;
    width: 220px;
    background-color: var(--color-surface-container-high);
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 12px rgba(0,0,0,0.5);
    z-index: 10;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .popover-header {
    padding: 12px 16px;
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    color: var(--color-outline);
    border-bottom: 1px solid var(--color-outline-variant);
    text-transform: uppercase;
  }

  .popover-list {
    display: flex;
    flex-direction: column;
    max-height: 200px;
    overflow-y: auto;
  }

  .popover-item {
    padding: 10px 16px;
    text-align: left;
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    color: var(--color-on-surface);
    transition: background-color var(--transition-fast);
  }

  .popover-item:hover {
    background-color: var(--color-surface-container-highest);
  }

  .popover-empty {
    padding: 16px;
    text-align: center;
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    color: var(--color-outline);
  }

  /* Badges */
  .badge-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .badge {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    font-weight: var(--font-mono-label-weight);
    letter-spacing: var(--font-mono-label-letter-spacing);
    padding: 4px 10px;
    border-radius: var(--radius-pill);
  }

  .badge.filled {
    background-color: var(--color-primary-container);
    color: var(--color-on-primary-container);
  }

  .badge.outline {
    border: 1px solid var(--color-outline-variant);
    color: var(--color-on-surface-variant);
  }

  /* Tabs & Queue */
  .tabs-section {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    margin-top: var(--space-2);
  }

  .tabs-header {
    display: flex;
    align-items: center;
    gap: var(--gutter);
    border-bottom: 1px solid var(--color-outline-variant);
    margin-bottom: var(--margin-sm);
  }

  .tab-btn {
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    letter-spacing: var(--font-nav-caps-letter-spacing);
    font-weight: var(--font-nav-caps-weight);
    color: var(--color-outline);
    padding: 8px 0;
    border-bottom: 2px solid transparent;
    transition: all var(--transition-fast);
  }

  .tab-btn:hover {
    color: var(--color-on-surface-variant);
  }

  .tab-btn.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding-right: 8px; /* scrollbar padding */
  }

  .tab-content::-webkit-scrollbar {
    width: 6px;
  }
  .tab-content::-webkit-scrollbar-thumb {
    background-color: var(--color-outline-variant);
    border-radius: 3px;
  }

  /* Queue List */
  .queue-list {
    display: flex;
    flex-direction: column;
  }

  .queue-row {
    display: flex;
    align-items: center;
    padding: 10px 12px;
    border-radius: var(--radius-lg);
    gap: var(--space-2);
    text-align: left;
    transition: background-color var(--transition-fast);
  }

  .queue-row:hover {
    background-color: var(--color-surface-container);
  }

  .queue-row.active-row {
    background-color: var(--color-surface-container-high);
  }

  .queue-index {
    width: 24px;
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    color: var(--color-outline);
    text-align: center;
    flex-shrink: 0;
  }

  :global(.playing-icon) {
    color: var(--color-primary);
  }

  .queue-meta {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .queue-title {
    font-family: var(--font-headline-sm-family);
    font-size: var(--font-headline-sm-size);
    font-weight: var(--font-headline-sm-weight);
    color: var(--color-on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .active-row .queue-title {
    color: var(--color-primary);
  }

  .queue-artist {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    color: var(--color-on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .queue-duration {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    color: var(--color-outline);
  }

  .queue-empty {
    padding: var(--margin-lg) 0;
    text-align: center;
    font-family: var(--font-body-sm-family);
    color: var(--color-outline);
  }

  /* Lyrics Empty */
  .lyrics-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 200px;
  }

  .empty-text {
    font-family: var(--font-body-md-family);
    font-size: var(--font-body-md-size);
    color: var(--color-on-surface-variant);
  }
</style>
