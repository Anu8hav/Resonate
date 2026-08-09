<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let checked: boolean = false;
  export let disabled: boolean = false;

  const dispatch = createEventDispatcher<{ change: boolean }>();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    dispatch('change', checked);
  }
</script>

<button
  class="toggle-switch"
  class:active={checked}
  class:disabled
  role="switch"
  aria-checked={checked}
  on:click={toggle}
>
  <span class="track"></span>
  <span class="knob"></span>
</button>

<style>
  .toggle-switch {
    position: relative;
    width: 40px;
    height: 22px;
    padding: 0;
    border-radius: var(--radius-pill);
    cursor: pointer;
    flex-shrink: 0;
    transition: all var(--transition-normal);
  }

  .toggle-switch.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .track {
    position: absolute;
    inset: 0;
    border-radius: var(--radius-pill);
    border: 1.5px solid var(--color-outline-variant);
    background-color: transparent;
    transition: all var(--transition-normal);
  }

  .active .track {
    border-color: var(--color-primary-container);
    background-color: var(--color-primary-container);
  }

  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background-color: var(--color-outline);
    transition: all var(--transition-normal);
  }

  .active .knob {
    left: 21px;
    background-color: var(--color-on-primary-container);
  }
</style>
