<script lang="ts">
  import { tracks } from '$lib/stores/library';
  import { currentTrack, play, formatTime } from '$lib/stores/player';
  import { Search, Volume2 } from 'lucide-svelte';

  let searchQuery = '';

  $: filteredTracks = searchQuery
    ? $tracks.filter(
        (t) =>
          t.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.artist.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.album.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : $tracks;
</script>

<div class="tracks-page">
  <!-- Page Header -->
  <div class="page-header">
    <h1 class="page-title">Tracks</h1>
    <div class="header-actions">
      <div class="search-wrapper">
        <Search size={16} />
        <input
          type="text"
          placeholder="Search tracks…"
          class="search-input"
          bind:value={searchQuery}
        />
      </div>
    </div>
  </div>

  <!-- Tracks Table -->
  {#if filteredTracks.length > 0}
    <div class="table-container">
      <table class="tracks-table">
        <thead>
          <tr>
            <th class="col-num">#</th>
            <th class="col-title">TITLE</th>
            <th class="col-artist">ARTIST</th>
            <th class="col-album">ALBUM</th>
            <th class="col-time">TIME</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredTracks as track, i (track.id)}
            {@const isCurrent = $currentTrack?.id === track.id}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
            <tr class:playing={isCurrent} on:click={() => play(track)}>
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
              <td class="col-artist">{track.artist}</td>
              <td class="col-album">{track.album}</td>
              <td class="col-time">{formatTime(track.durationSeconds)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-text">No tracks found</span>
    </div>
  {/if}
</div>

<style>
  .tracks-page {
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
  .col-title { width: 35%; }
  .col-artist { width: 22%; }
  .col-album { width: 25%; }
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

  .col-artist {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    color: var(--color-on-surface-variant);
  }

  .col-album {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    color: var(--color-on-surface-variant);
  }

  td.col-time {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    font-weight: var(--font-mono-data-weight);
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
