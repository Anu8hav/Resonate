<script lang="ts">
  import { goto } from '$app/navigation';
  import { CloudOff } from 'lucide-svelte';
</script>

<div class="subsonic-page">
  <!-- Page Header -->
  <div class="page-header">
    <h1 class="page-title">Subsonic</h1>
  </div>

  <!-- Disconnected State -->
  <div class="disconnected-state">
    <div class="state-icon">
      <CloudOff size={48} />
    </div>
    <h2 class="state-heading">Not connected to a Subsonic server</h2>
    <p class="state-body">Configure a server connection to browse your remote library.</p>
    <button class="settings-btn" on:click={() => goto('/settings')}>
      GO TO SERVER SETTINGS
    </button>
  </div>
</div>

<style>
  .subsonic-page {
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

  /* ── Disconnected State ────────────────────────────────────────── */

  .disconnected-state {
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

  .settings-btn {
    margin-top: var(--space-2);
    padding: 8px 20px;
    border: 1px solid var(--color-outline-variant);
    border-radius: var(--radius-lg);
    color: var(--color-primary);
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    letter-spacing: var(--font-nav-caps-letter-spacing);
    font-weight: var(--font-nav-caps-weight);
    text-transform: uppercase;
    transition: all var(--transition-fast);
  }

  .settings-btn:hover {
    border-color: var(--color-primary);
    background-color: var(--color-surface-container);
  }
</style>
