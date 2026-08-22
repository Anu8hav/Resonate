<script lang="ts">
  import { albums } from '$lib/stores/library';
  import { Search, Bell, User, ChevronDown } from 'lucide-svelte';
  import { goto } from '$app/navigation';

  let searchQuery = '';

  $: filteredAlbums = searchQuery
    ? $albums.filter(
        (a) =>
          a.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          a.artist.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : $albums;
</script>

<div class="albums-page">
  <!-- Page Header -->
  <div class="page-header">
    <h1 class="page-title">Albums</h1>
    <div class="header-actions">
      <div class="search-wrapper">
        <Search size={16} />
        <input
          type="text"
          placeholder="Search albums…"
          class="search-input"
          bind:value={searchQuery}
        />
      </div>
      <button class="icon-action" title="Notifications">
        <Bell size={18} />
      </button>
      <button class="icon-action" title="Account">
        <User size={18} />
      </button>
    </div>
  </div>

  <!-- Sort / Count Bar -->
  <div class="toolbar">
    <button class="sort-control">
      <span class="sort-label">SORT BY:</span>
      <span class="sort-value">RECENT</span>
      <ChevronDown size={14} />
    </button>
    <span class="total-count">{filteredAlbums.length.toLocaleString()} ALBUMS</span>
  </div>

  <!-- Album Grid -->
  {#if filteredAlbums.length > 0}
    <div class="album-grid">
      {#each filteredAlbums as album (album.id)}
        {@const isSingle = album.totalTracks === null || album.totalTracks === 1}
        {@const badgeText = isSingle ? 'SINGLE' : (album.locallyOwnedCount < (album.totalTracks || 0) ? `${album.locallyOwnedCount}/${album.totalTracks} TRACKS` : null)}
        <button class="album-card" on:click={() => goto(`/albums/${album.id}`)}>
          <div class="card-cover">
            {#if badgeText}
              <div class="album-badge">{badgeText}</div>
            {/if}
            {#if album.coverUrl}
              <img src={album.coverUrl} alt={album.title} class="cover-img" />
            {:else}
              <div class="cover-placeholder">
                <span class="no-cover-text">NO COVER</span>
              </div>
            {/if}
          </div>
          <div class="card-meta">
            <span class="card-title">{album.title}</span>
            <span class="card-artist">{album.artist}</span>
          </div>
        </button>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-text">No albums found</span>
    </div>
  {/if}
</div>

<style>
  .albums-page {
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
    min-height: 40px;
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

  .icon-action {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-lg);
    color: var(--color-on-surface-variant);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .icon-action:hover {
    background-color: var(--color-surface-container-high);
    color: var(--color-on-surface);
  }

  /* ── Toolbar (Sort + Count) ────────────────────────────────────── */

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sort-control {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: var(--radius-default);
    color: var(--color-on-surface-variant);
    transition: color var(--transition-fast);
  }

  .sort-control:hover {
    color: var(--color-on-surface);
  }

  .sort-label {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    text-transform: uppercase;
    color: var(--color-outline);
  }

  .sort-value {
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    line-height: var(--font-nav-caps-line-height);
    letter-spacing: var(--font-nav-caps-letter-spacing);
    font-weight: var(--font-nav-caps-weight);
    text-transform: uppercase;
  }

  .total-count {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    line-height: var(--font-mono-data-line-height);
    font-weight: var(--font-mono-data-weight);
    color: var(--color-on-surface-variant);
  }

  /* ── Album Grid ────────────────────────────────────────────────── */

  .album-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--gutter);
  }

  .album-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    text-align: left;
    padding: 0;
    border-radius: var(--radius-default);
    transition: all var(--transition-fast);
    cursor: pointer;
    background: transparent;
    border: none;
    outline: none;
  }

  .album-card:hover .card-cover {
    border-color: var(--color-primary);
    transform: scale(1.02);
  }
  
  .album-card:active .card-cover {
    transform: scale(0.98);
  }

  .card-cover {
    position: relative;
    aspect-ratio: 1;
    border-radius: var(--radius-default);
    overflow: hidden;
    border: 1px solid transparent;
    transition: border-color var(--transition-fast);
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
    /* ensure it doesn't cramp */
    max-width: calc(100% - 16px);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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

  .no-cover-text {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-outline);
    text-transform: uppercase;
  }

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

  .card-artist {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    line-height: var(--font-body-sm-line-height);
    color: var(--color-on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ── Empty State ───────────────────────────────────────────────── */

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--margin-lg) 0;
  }

  .empty-text {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-body-md-size);
    color: var(--color-outline);
  }
</style>
