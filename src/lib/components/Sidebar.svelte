<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  import {
    Library,
    Disc3,
    Users,
    Music,
    ListMusic,
    Server,
    Radio,
    History
  } from 'lucide-svelte';

  interface NavItem {
    label: string;
    href: string;
    icon: typeof Library;
  }

  const primaryNav: NavItem[] = [
    { label: 'Library', href: '/', icon: Library },
    { label: 'Albums', href: '/albums', icon: Disc3 },
    { label: 'Artists', href: '/artists', icon: Users },
    { label: 'Tracks', href: '/tracks', icon: Music },
    { label: 'Playlists', href: '/playlists', icon: ListMusic },
  ];

  const systemNav: NavItem[] = [
    { label: 'Server Settings', href: '/settings', icon: Server },
    { label: 'Subsonic', href: '/subsonic', icon: Radio },
    { label: 'History', href: '/history', icon: History },
  ];

  function navigate(href: string) {
    goto(href);
  }

  function isActive(pathname: string, href: string): boolean {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  }
</script>

<aside class="sidebar" data-tauri-drag-region>
  <!-- Brand -->
  <div class="brand" data-tauri-drag-region>
    <span class="wordmark">RESONATE</span>
    <span class="version">V1.0.0</span>
  </div>

  <!-- Primary Navigation -->
  <nav class="nav-section">
    {#each primaryNav as item}
      <button
        class="nav-item"
        class:active={isActive($page.url.pathname, item.href)}
        on:click={() => navigate(item.href)}
      >
        <svelte:component this={item.icon} size={16} />
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- Divider -->
  <div class="divider"></div>

  <!-- System Section -->
  <div class="section-header">SYSTEM</div>
  <nav class="nav-section">
    {#each systemNav as item}
      <button
        class="nav-item"
        class:active={isActive($page.url.pathname, item.href)}
        on:click={() => navigate(item.href)}
      >
        <svelte:component this={item.icon} size={16} />
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="spacer"></div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background-color: var(--color-surface-container-lowest);
    display: flex;
    flex-direction: column;
    padding: var(--gutter);
    padding-top: 0;
    user-select: none;
    overflow-y: auto;
    overflow-x: hidden;
    flex-shrink: 0;
  }

  .brand {
    padding: var(--margin-md) 0 var(--margin-md) 0;
    -webkit-app-region: drag;
  }



  .wordmark {
    font-family: var(--font-display);
    font-size: 18px;
    font-weight: 600;
    letter-spacing: 0.12em;
    color: var(--color-on-surface);
  }

  .version {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-on-surface-variant);
    text-transform: uppercase;
    margin-top: 2px;
  }

  .nav-section {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--margin-sm);
    padding: 8px 12px;
    border-radius: var(--radius-lg);
    color: var(--color-on-surface-variant);
    font-family: var(--font-nav-caps-family);
    font-size: var(--font-nav-caps-size);
    line-height: var(--font-nav-caps-line-height);
    letter-spacing: var(--font-nav-caps-letter-spacing);
    font-weight: var(--font-nav-caps-weight);
    text-transform: uppercase;
    transition: background-color var(--transition-fast),
                color var(--transition-fast);
    -webkit-app-region: no-drag;
  }

  .nav-item:hover {
    background-color: var(--color-surface-container);
    color: var(--color-on-surface);
  }

  .nav-item.active {
    background-color: var(--color-surface-container-high);
    color: var(--color-primary);
  }

  .nav-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .divider {
    height: 1px;
    background-color: var(--color-outline-variant);
    margin: var(--margin-sm) 0;
  }

  .section-header {
    font-family: var(--font-mono-label-family);
    font-size: var(--font-mono-label-size);
    line-height: var(--font-mono-label-line-height);
    letter-spacing: var(--font-mono-label-letter-spacing);
    font-weight: var(--font-mono-label-weight);
    color: var(--color-outline);
    text-transform: uppercase;
    padding: var(--space-2) 12px;
    margin-bottom: 2px;
  }

  .spacer {
    flex: 1;
  }
</style>
