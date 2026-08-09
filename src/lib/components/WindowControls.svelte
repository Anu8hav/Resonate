<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Minus, Square, X } from 'lucide-svelte';

  async function minimize() {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.minimize();
    } catch (e) {
      console.log('Window minimize simulated');
    }
  }

  async function toggleMaximize() {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.toggleMaximize();
    } catch (e) {
      console.log('Window maximize simulated');
    }
  }

  async function closeWindow() {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.close();
    } catch (e) {
      console.log('Window close simulated');
    }
  }
</script>

<div class="window-controls">
  <button class="control-btn" on:click={minimize} title="Minimize">
    <Minus size={14} />
  </button>
  <button class="control-btn" on:click={toggleMaximize} title="Maximize">
    <Square size={11} />
  </button>
  <button class="control-btn close" on:click={closeWindow} title="Close">
    <X size={14} />
  </button>
</div>

<style>
  .window-controls {
    display: flex;
    align-items: center;
    gap: 0;
    -webkit-app-region: no-drag;
    height: 100%;
  }

  .control-btn {
    color: var(--color-on-surface-variant);
    width: 46px;
    height: 100%;
    border-radius: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color var(--transition-fast),
                color var(--transition-fast);
    -webkit-app-region: no-drag;
  }

  .control-btn:hover {
    background-color: var(--color-surface-container-high);
    color: var(--color-on-surface);
  }

  .control-btn.close:hover {
    background-color: #e81123;
    color: #ffffff;
  }
</style>
