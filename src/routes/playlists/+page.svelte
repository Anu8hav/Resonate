<script lang="ts">
  import { playlists } from '$lib/stores/library';
  import { Search, Plus } from 'lucide-svelte';

  let searchQuery = '';

  $: filteredPlaylists = searchQuery
    ? $playlists.filter(
        (p) => p.title.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : $playlists;
</script>

<div class="playlists-page">
  <!-- Page Header -->
  <div class="page-header">
    <h1 class="page-title">Playlists</h1>
    <div class="header-actions">
      <div class="search-wrapper">
        <Search size={16} />
        <input
          type="text"
          placeholder="Search playlists…"
          class="search-input"
          bind:value={searchQuery}
        />
      </div>
      <button class="new-playlist-btn">
        <Plus size={14} />
        <span>NEW PLAYLIST</span>
      </button>
    </div>
  </div>

  <!-- Count Bar -->
  <div class="toolbar">
    <span class="total-count">{filteredPlaylists.length} PLAYLISTS</span>
  </div>

  <!-- Playlist Grid -->
  {#if filteredPlaylists.length > 0}
    <div class="playlist-grid">
      {#each filteredPlaylists as playlist (playlist.id)}
        <button class="playlist-card">
          <div class="card-cover">
            {#if playlist.coverUrl}
              <img src={playlist.coverUrl} alt={playlist.title} class="cover-img" />
            {:else}
              <div class="cover-placeholder">
                <div class="stacked-tracks">
                  <div class="stack-line"></div>
                  <div class="stack-line"></div>
                  <div class="stack-line"></div>
                </div>
              </div>
            {/if}
          </div>
          <div class="card-meta">
            <span class="card-title">{playlist.title}</span>
            <span class="card-subtitle">{playlist.trackIds.length} {playlist.trackIds.length === 1 ? 'track' : 'tracks'}</span>
          </div>
        </button>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-text">No playlists yet</span>
    </div>
  {/if}
</div>

<style>
  .playlists-page {
    display: flex;
    flex-direction: column;
    gap: var(--gutter);
    max-width: var(--max-content-width);
  }

  /* ── Page Header ───────────────────────────────────────────────── */

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--gutter);
  }

  .page-title {
    font-family: var(--font-headline-lg-family);
    font-size: var(--font-headline-lg-size);
    line-height: var(--font-headline-lg-line-height);
    letter-spacing: var(--font-headline-lg-letter-spacing);
    font-weight: var(--font-headline-lg-weight);
    color: var(--color-on-surface);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .search-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 6px 12px;
    background-color: var(--color-surface-container);
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-lg);
    color: var(--color-outline);
    transition: border-color var(--transition-fast);
  }

  .search-wrapper:focus-within {
    border-color: var(--color-primary);
  }

  .search-input {
    background: none;
    border: none;
    outline: none;
    color: var(--color-on-surface);
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    width: 180px;
  }

  .search-input::placeholder {
    color: var(--color-outline);
  }

  .new-playlist-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-lg);
    color: var(--color-primary);
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    text-transform: uppercase;
    transition: all var(--transition-fast);
  }

  .new-playlist-btn:hover {
    border-color: var(--color-primary);
    background-color: var(--color-surface-container);
  }

  /* ── Toolbar ────────────────────────────────────────────────────── */

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
  }

  .total-count {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    line-height: var(--font-mono-data-line-height);
    font-weight: var(--font-mono-data-weight);
    color: var(--color-on-surface-variant);
  }

  /* ── Playlist Grid ─────────────────────────────────────────────── */

  .playlist-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--gutter);
  }

  .playlist-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    text-align: left;
    padding: 0;
    border-radius: var(--radius-default);
    transition: background-color var(--transition-fast);
  }

  .playlist-card:hover .card-cover {
    border-color: var(--color-outline-variant);
  }

  .card-cover {
    aspect-ratio: 1;
    border-radius: var(--radius-default);
    overflow: hidden;
    border: 1px solid transparent;
    transition: border-color var(--transition-fast);
  }

  .cover-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-placeholder {
    width: 100%;
    height: 100%;
    background-color: var(--color-surface-container);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stacked-tracks {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 40%;
  }

  .stack-line {
    height: 3px;
    border-radius: 1px;
    background-color: var(--color-outline-variant);
  }

  .stack-line:nth-child(1) { width: 100%; }
  .stack-line:nth-child(2) { width: 75%; }
  .stack-line:nth-child(3) { width: 50%; }

  .card-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 2px;
  }

  .card-title {
    font-family: var(--font-headline-sm-family);
    font-size: var(--font-headline-sm-size);
    line-height: var(--font-headline-sm-line-height);
    letter-spacing: var(--font-headline-sm-letter-spacing);
    font-weight: var(--font-headline-sm-weight);
    color: var(--color-on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-subtitle {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    line-height: var(--font-body-sm-line-height);
    color: var(--color-on-surface-variant);
  }

  /* ── Empty State ───────────────────────────────────────────────── */

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--margin-lg) 0;
  }

  .empty-text {
    font-family: var(--font-body-md-family);
    font-size: var(--font-body-md-size);
    line-height: var(--font-body-md-line-height);
    color: var(--color-on-surface-variant);
  }
</style>
