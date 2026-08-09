<script lang="ts">
  import { onMount } from 'svelte';
  import '$lib/theme/fonts.css';
  import '$lib/theme/tokens.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import NowPlayingBar from '$lib/components/NowPlayingBar.svelte';
  import NowPlayingExpanded from '$lib/components/NowPlayingExpanded.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';
  import { isExpandedViewOpen } from '$lib/stores/player';
  import { refreshLibraryFromBackend } from '$lib/stores/library';

  // Load persisted library from SQLite on app startup
  onMount(() => {
    refreshLibraryFromBackend();
  });
</script>

<div class="app-shell">
  <div class="titlebar" data-tauri-drag-region>
    <WindowControls />
  </div>
  <div class="app-body">
    <Sidebar />
    <main class="main-content">
      <slot />
    </main>
  </div>
  {#if !$isExpandedViewOpen}
    <NowPlayingBar />
  {/if}
  <NowPlayingExpanded />
</div>

<style>
  .app-shell {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background-color: var(--color-background);
  }

  .titlebar {
    width: 100%;
    height: var(--titlebar-height, 32px);
    min-height: var(--titlebar-height, 32px);
    display: flex;
    align-items: center;
    justify-content: flex-end;
    background-color: var(--color-surface-container-lowest);
    -webkit-app-region: drag;
    user-select: none;
    flex-shrink: 0;
    z-index: 1000;
  }

  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--margin-md);
    background-color: var(--color-surface);
  }
</style>
