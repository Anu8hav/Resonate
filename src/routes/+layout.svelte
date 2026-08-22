<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { page } from '$app/stores';
  import { fly, fade } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import '$lib/theme/fonts.css';
  import '$lib/theme/tokens.css';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import NowPlayingBar from '$lib/components/NowPlayingBar.svelte';
  import NowPlayingExpanded from '$lib/components/NowPlayingExpanded.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';
  import { isExpandedViewOpen, playbackState, handleTrackEnd, togglePlay, currentTrack } from '$lib/stores/player';
  import { settingsState } from '$lib/stores/server';
  import { extractColorTheme } from '$lib/utils/color';
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

  let lastExtractedCoverUrl: string | null = null;

  $: if (typeof document !== 'undefined') {
    if ($currentTrack?.coverUrl !== lastExtractedCoverUrl) {
      lastExtractedCoverUrl = $currentTrack?.coverUrl ?? null;
      if (lastExtractedCoverUrl) {
        extractColorTheme(lastExtractedCoverUrl).then(({ ambient, accent, accentDark }) => {
          document.documentElement.style.setProperty('--color-ambient-wash', ambient);
          if ($settingsState.dynamicAccentColor) {
            document.documentElement.style.setProperty('--color-primary-dynamic', accent);
            document.documentElement.style.setProperty('--color-primary-container-dynamic', accentDark);
          }
        }).catch(err => {
          console.error("Color extraction failed:", err);
          document.documentElement.style.setProperty('--color-ambient-wash', 'var(--color-primary-container)');
        });
      } else {
        document.documentElement.style.removeProperty('--color-primary-dynamic');
        document.documentElement.style.removeProperty('--color-primary-container-dynamic');
        document.documentElement.style.removeProperty('--color-ambient-wash');
      }
    }
  }

  // Handle setting toggle independently
  $: if (typeof document !== 'undefined') {
    if (!$settingsState.dynamicAccentColor) {
      document.documentElement.style.removeProperty('--color-primary-dynamic');
      document.documentElement.style.removeProperty('--color-primary-container-dynamic');
    } else if (lastExtractedCoverUrl) {
      extractColorTheme(lastExtractedCoverUrl).then(({ accent, accentDark }) => {
        document.documentElement.style.setProperty('--color-primary-dynamic', accent);
        document.documentElement.style.setProperty('--color-primary-container-dynamic', accentDark);
      }).catch(() => {});
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
      {#key $page.url.pathname}
        <div in:fly={{ y: 8, duration: 200, easing: cubicOut }} out:fade={{ duration: 150 }} style="height: 100%;">
          <slot />
        </div>
      {/key}
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
