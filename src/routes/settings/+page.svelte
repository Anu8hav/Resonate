<script lang="ts">
  import { serverConfig, settingsState } from '$lib/stores/server';
  import ToggleSwitch from '$lib/components/ToggleSwitch.svelte';
  import { RefreshCw } from 'lucide-svelte';

  // Sub-nav state for settings context
  type SettingsTab = 'server';
  let activeSettingsTab: SettingsTab = 'server';

  function handleTranscodeToggle(e: CustomEvent<boolean>) {
    settingsState.update((s) => ({ ...s, transcodeFLACToOpus: e.detail }));
  }

  function handleScrobbleToggle(e: CustomEvent<boolean>) {
    settingsState.update((s) => ({ ...s, scrobbleToLastFm: e.detail }));
  }

  $: cachePercent = ($settingsState.cacheUsedGB / $settingsState.cacheTotalGB) * 100;
</script>

<div class="settings-page">
  <!-- Page Header -->
  <div class="page-header">
    <h1 class="page-title">Server Settings</h1>
  </div>

  <!-- Sub-nav -->
  <div class="sub-nav">
    <button class="sub-nav-item active">Server Settings</button>
    <button class="sub-nav-item">Library</button>
    <button class="sub-nav-item">Playlists</button>
    <button class="sub-nav-item">Artists</button>
  </div>

  <!-- Panel: Connection Details -->
  <div class="panel">
    <div class="panel-header">
      <h2 class="panel-title">Connection Details</h2>
      <span class="api-badge">API v{$serverConfig.apiVersion}</span>
    </div>

    <div class="field-group">
      <label class="field-label" for="server-url">SERVER URL</label>
      <input
        id="server-url"
        type="text"
        class="field-input"
        value={$serverConfig.url}
        readonly
      />
    </div>

    <div class="field-row">
      <div class="field-group">
        <label class="field-label" for="server-username">USERNAME</label>
        <input
          id="server-username"
          type="text"
          class="field-input"
          value={$serverConfig.username}
          readonly
        />
      </div>
      <div class="field-group">
        <label class="field-label" for="server-token">API TOKEN / PASSWORD</label>
        <input
          id="server-token"
          type="password"
          class="field-input"
          value={$serverConfig.apiToken}
          readonly
        />
      </div>
    </div>
  </div>

  <!-- Panel: Diagnostics -->
  <div class="panel">
    <div class="panel-header">
      <h2 class="panel-title">Diagnostics</h2>
      <span class="status-pill" class:online={$serverConfig.connectionStatus === 'online'}>
        <span class="status-dot"></span>
        <span class="status-text">
          {$serverConfig.connectionStatus === 'online' ? 'ONLINE' : $serverConfig.connectionStatus === 'offline' ? 'OFFLINE' : 'CHECKING'}
        </span>
      </span>
    </div>

    <div class="diag-grid">
      <div class="diag-item">
        <span class="diag-label">LATENCY</span>
        <span class="diag-value">{$serverConfig.latencyMs}ms</span>
      </div>
      <div class="diag-item">
        <span class="diag-label">LAST SYNC</span>
        <span class="diag-value">{$serverConfig.lastSyncAt}</span>
      </div>
      <div class="diag-item diag-action">
        <button class="resync-btn">
          <RefreshCw size={14} />
          <span>Force Full Resync</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Panel: Advanced Playback -->
  <div class="panel">
    <div class="panel-header">
      <h2 class="panel-title">Advanced Playback</h2>
    </div>

    <!-- Transcode Toggle -->
    <div class="toggle-row">
      <div class="toggle-info">
        <span class="toggle-label">Transcode FLAC to Opus</span>
        <span class="toggle-desc">Re-encode lossless files for lower bandwidth streaming</span>
      </div>
      <ToggleSwitch
        checked={$settingsState.transcodeFLACToOpus}
        on:change={handleTranscodeToggle}
      />
    </div>

    <!-- Scrobble Toggle -->
    <div class="toggle-row">
      <div class="toggle-info">
        <span class="toggle-label">Scrobble to Last.fm</span>
        <span class="toggle-desc">Send listening history to your Last.fm profile</span>
      </div>
      <ToggleSwitch
        checked={$settingsState.scrobbleToLastFm}
        on:change={handleScrobbleToggle}
      />
    </div>

    <!-- Cache Bar -->
    <div class="cache-section">
      <div class="cache-header">
        <span class="toggle-label">Local Audio Cache</span>
        <button class="clear-cache-btn">CLEAR CACHE</button>
      </div>
      <div class="cache-bar-track">
        <div class="cache-bar-fill" style="width: {cachePercent}%"></div>
      </div>
      <span class="cache-label">{$settingsState.cacheUsedGB.toFixed(1)} GB / {$settingsState.cacheTotalGB.toFixed(1)} GB</span>
    </div>
  </div>
</div>

<style>
  .settings-page {
    display: flex;
    flex-direction: column;
    gap: var(--gutter);
    max-width: 720px;
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

  /* ── Sub-nav ───────────────────────────────────────────────────── */

  .sub-nav {
    display: flex;
    gap: var(--space-2);
  }

  .sub-nav-item {
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    line-height: var(--font-nav-caps-line-height);
    letter-spacing: var(--font-nav-caps-letter-spacing);
    font-weight: var(--font-nav-caps-weight);
    text-transform: uppercase;
    color: var(--color-on-surface-variant);
    padding: 6px 12px;
    border-radius: var(--radius-lg);
    transition: all var(--transition-fast);
  }

  .sub-nav-item:hover {
    background-color: var(--color-surface-container);
  }

  .sub-nav-item.active {
    background-color: var(--color-surface-container-high);
    color: var(--color-primary);
  }

  /* ── Panels ────────────────────────────────────────────────────── */

  .panel {
    background-color: var(--color-surface-container);
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-xl);
    padding: var(--margin-md);
    display: flex;
    flex-direction: column;
    gap: var(--gutter);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-title {
    font-family: var(--font-headline-sm-family);
    font-size: var(--font-headline-sm-size);
    line-height: var(--font-headline-sm-line-height);
    letter-spacing: var(--font-headline-sm-letter-spacing);
    font-weight: var(--font-headline-sm-weight);
    color: var(--color-on-surface);
  }

  .api-badge {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-on-surface-variant);
    background-color: var(--color-surface-container-high);
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    text-transform: uppercase;
  }

  /* ── Fields ────────────────────────────────────────────────────── */

  .field-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-unit);
    flex: 1;
  }

  .field-label {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-outline);
    text-transform: uppercase;
  }

  .field-input {
    background-color: var(--color-surface-container-lowest);
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-default);
    padding: 8px 12px;
    color: var(--color-on-surface);
    font-family: var(--font-body-md-family);
    font-size: var(--font-body-md-size);
    outline: none;
    transition: border-color var(--transition-fast);
  }

  .field-input:focus {
    border-color: var(--color-primary);
  }

  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--gutter);
  }

  /* ── Diagnostics ───────────────────────────────────────────────── */

  .status-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    background-color: var(--color-surface-container-high);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: var(--color-outline);
  }

  .status-pill.online .status-dot {
    background-color: var(--color-status-online);
  }

  .status-text {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-on-surface-variant);
    text-transform: uppercase;
  }

  .diag-grid {
    display: flex;
    align-items: center;
    gap: var(--margin-md);
    flex-wrap: wrap;
  }

  .diag-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .diag-label {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-outline);
    text-transform: uppercase;
  }

  .diag-value {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    line-height: var(--font-mono-data-line-height);
    font-weight: var(--font-mono-data-weight);
    color: var(--color-on-surface);
  }

  .diag-action {
    margin-left: auto;
  }

  .resync-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-outline-variant);
    color: var(--color-on-surface-variant);
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    letter-spacing: var(--font-nav-caps-letter-spacing);
    font-weight: var(--font-nav-caps-weight);
    text-transform: uppercase;
    transition: all var(--transition-fast);
  }

  .resync-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  /* ── Toggle Rows ───────────────────────────────────────────────── */

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--gutter);
    padding: var(--space-2) 0;
  }

  .toggle-row + .toggle-row {
    border-top: 1px solid var(--color-surface-container-highest);
  }

  .toggle-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-label {
    font-family: var(--font-body-md-family);
    font-size: var(--font-body-md-size);
    line-height: var(--font-body-md-line-height);
    font-weight: 400;
    color: var(--color-on-surface);
  }

  .toggle-desc {
    font-family: var(--font-body-sm-family);
    font-size: var(--font-body-sm-size);
    line-height: var(--font-body-sm-line-height);
    color: var(--color-on-surface-variant);
  }

  /* ── Cache Section ─────────────────────────────────────────────── */

  .cache-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--color-surface-container-highest);
  }

  .cache-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .clear-cache-btn {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-primary);
    text-transform: uppercase;
    transition: color var(--transition-fast);
  }

  .clear-cache-btn:hover {
    color: var(--color-primary-container);
  }

  .cache-bar-track {
    height: 4px;
    background-color: var(--color-surface-container-highest);
    border-radius: 2px;
    overflow: hidden;
  }

  .cache-bar-fill {
    height: 100%;
    background-color: var(--color-primary);
    border-radius: 2px;
    transition: width var(--transition-normal);
  }

  .cache-label {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    line-height: var(--font-mono-data-line-height);
    font-weight: var(--font-mono-data-weight);
    color: var(--color-on-surface-variant);
  }
</style>
