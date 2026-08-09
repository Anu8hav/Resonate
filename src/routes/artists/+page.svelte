<script lang="ts">
  import { artists } from '$lib/stores/library';
  import { Search } from 'lucide-svelte';

  let searchQuery = '';

  $: filteredArtists = searchQuery
    ? $artists.filter(
        (a) => a.name.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : $artists;
</script>

<div class="artists-page">
  <!-- Page Header -->
  <div class="page-header">
    <h1 class="page-title">Artists</h1>
    <div class="header-actions">
      <div class="search-wrapper">
        <Search size={16} />
        <input
          type="text"
          placeholder="Search artists…"
          class="search-input"
          bind:value={searchQuery}
        />
      </div>
    </div>
  </div>

  <!-- Count Bar -->
  <div class="toolbar">
    <span class="total-count">{filteredArtists.length} ARTISTS</span>
  </div>

  <!-- Artist Grid -->
  {#if filteredArtists.length > 0}
    <div class="artist-grid">
      {#each filteredArtists as artist (artist.name)}
        <button class="artist-card">
          <div class="artist-avatar">
            {#if artist.imageUrl}
              <img src={artist.imageUrl} alt={artist.name} class="avatar-img" />
            {:else}
              <div class="avatar-placeholder">
                <span class="no-image-text">NO IMAGE</span>
              </div>
            {/if}
          </div>
          <div class="card-meta">
            <span class="card-title">{artist.name}</span>
            <span class="card-subtitle">{artist.albumCount} {artist.albumCount === 1 ? 'album' : 'albums'}</span>
          </div>
        </button>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-text">No artists found</span>
    </div>
  {/if}
</div>

<style>
  .artists-page {
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

  /* ── Artist Grid ───────────────────────────────────────────────── */

  .artist-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--gutter);
  }

  .artist-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    text-align: center;
    padding: var(--margin-sm);
    border-radius: var(--radius-lg);
    transition: background-color var(--transition-fast);
  }

  .artist-card:hover {
    background-color: var(--color-surface-container);
  }

  .artist-avatar {
    width: 120px;
    height: 120px;
    border-radius: 50%;
    overflow: hidden;
    border: 1px solid transparent;
    transition: border-color var(--transition-fast);
    flex-shrink: 0;
  }

  .artist-card:hover .artist-avatar {
    border-color: var(--color-outline-variant);
  }

  .avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-placeholder {
    width: 100%;
    height: 100%;
    background-color: var(--color-surface-container);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .no-image-text {
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
    font-family: var(--font-mono-data-family);
    font-size: var(--font-body-md-size);
    color: var(--color-outline);
  }
</style>
