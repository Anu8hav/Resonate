<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import '$lib/theme/fonts.css';
  import '$lib/theme/tokens.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import NowPlayingBar from '$lib/components/NowPlayingBar.svelte';
  import NowPlayingExpanded from '$lib/components/NowPlayingExpanded.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';
  import { isExpandedViewOpen, playbackState, handleTrackEnd, togglePlay } from '$lib/stores/player';
  import { refreshLibraryFromBackend } from '$lib/stores/library';
  import { listen } from '@tauri-apps/api/event';

  let unlistenPosition: UnlistenFn | undefined;
  let unlistenEnded: UnlistenFn | undefined;

  // Load persisted library from SQLite on app startup
  onMount(() => {
    refreshLibraryFromBackend();

    // Setup Tauri event listeners for audio playback
    listen<number>('playback-position', (event) => {
      playbackState.update(s => ({ ...s, positionSeconds: event.payload }));
    }).then(fn => { unlistenPosition = fn; }).catch(console.warn);

    listen('track-ended', () => {
      handleTrackEnd();
    }).then(fn => { unlistenEnded = fn; }).catch(console.warn);
  });

  onDestroy(() => {
    unlistenPosition?.();
    unlistenEnded?.();
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.code === 'Space') {
      const activeEl = document.activeElement;
      if (
        activeEl &&
        (activeEl.tagName === 'INPUT' ||
         activeEl.tagName === 'TEXTAREA' ||
         activeEl.hasAttribute('contenteditable'))
      ) {
        return;
      }
      event.preventDefault();
      togglePlay();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

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
