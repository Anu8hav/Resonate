<script lang="ts">
  import { history, play, currentTrack, formatTime } from '$lib/stores/player';
  import { Clock, Volume2 } from 'lucide-svelte';
</script>

<div class="history-page">
  <!-- Page Header -->
  <div class="page-header">
    <h1 class="page-title">History</h1>
  </div>

  {#if $history.length > 0}
    <!-- History Table -->
    <div class="table-container">
      <table class="tracks-table">
        <thead>
          <tr>
            <th class="col-icon"></th>
            <th class="col-title">TITLE</th>
            <th class="col-artist">ARTIST</th>
            <th class="col-album">ALBUM</th>
            <th class="col-time">TIME</th>
          </tr>
        </thead>
        <tbody>
          {#each $history as track (Math.random() + track.id)}
            {@const isCurrent = $currentTrack?.id === track.id}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
            <tr class:playing={isCurrent} on:click={() => play(track)}>
              <td class="col-icon">
                {#if isCurrent}
                  <span class="playing-icon"><Volume2 size={14} /></span>
                {:else}
                  <span class="history-icon" title="Played recently"><Clock size={14} /></span>
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
    <!-- Empty State -->
    <div class="empty-state">
      <div class="state-icon">
        <Clock size={48} />
      </div>
      <h2 class="state-heading">No listening history yet</h2>
      <p class="state-body">Tracks you play will show up here.</p>
    </div>
  {/if}
</div>

<style>
  .history-page {
    display: flex;
    flex-direction: column;
    gap: var(--gutter);
    max-width: var(--max-content-width);
    height: 100%;
  }

  /* ── Page Header ───────────────────────────────────────────────── */

  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .page-title {
    font-family: var(--font-headline-lg-family);
    font-size: var(--font-headline-lg-size);
    line-height: var(--font-headline-lg-line-height);
    letter-spacing: var(--font-headline-lg-letter-spacing);
    font-weight: var(--font-headline-lg-weight);
    color: var(--color-on-surface);
  }

  /* ── Empty State ───────────────────────────────────────────────── */

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--margin-sm);
    padding: var(--margin-lg) 0;
  }

  .state-icon {
    color: var(--color-outline);
    margin-bottom: var(--space-2);
  }

  .state-heading {
    font-family: var(--font-headline-sm-family);
    font-size: var(--font-headline-sm-size);
    line-height: var(--font-headline-sm-line-height);
    letter-spacing: var(--font-headline-sm-letter-spacing);
    font-weight: var(--font-headline-sm-weight);
    color: var(--color-on-surface);
    text-align: center;
  }

  .state-body {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    line-height: var(--font-body-sm-line-height);
    color: var(--color-on-surface-variant);
    text-align: center;
    max-width: 320px;
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
  .col-icon { width: 48px; text-align: center; }
  .col-title { width: 35%; }
  .col-artist { width: 22%; }
  .col-album { width: 25%; }
  .col-time { width: 72px; text-align: right; }

  .tracks-table th.col-icon,
  .tracks-table td.col-icon { text-align: center; }
  .tracks-table th.col-time,
  .tracks-table td.col-time { text-align: right; }

  /* Cell typography */
  .history-icon {
    color: var(--color-on-surface-variant);
    display: flex;
    align-items: center;
    justify-content: center;
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
</style>
