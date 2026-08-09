<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  // Root page redirects to Albums as the default view
  onMount(() => {
    goto('/albums', { replaceState: true });
  });
</script>

<div class="loading-placeholder">
  <span class="loading-text">Loading library…</span>
</div>

<style>
  .loading-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-outline);
  }

  .loading-text {
    font-family: var(--font-mono-data-family);
    font-size: var(--font-mono-data-size);
    font-weight: var(--font-mono-data-weight);
    letter-spacing: var(--font-mono-data-letter-spacing);
  }
</style>
