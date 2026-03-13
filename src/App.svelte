<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import KeyboardHandler from './lib/KeyboardHandler.svelte';
  import Sidebar from './lib/Sidebar.svelte';
  import MainPane from './lib/MainPane.svelte';
  import CommandPalette from './lib/CommandPalette.svelte';
  import ReminderPicker from './lib/ReminderPicker.svelte';
  import { refreshItems } from './lib/actions';

  let unlisteners: Array<() => void> = [];
  let debounceTimer: ReturnType<typeof setTimeout>;

  function debouncedRefresh() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => refreshItems(), 300);
  }

  onMount(async () => {
    await refreshItems();

    const unlisten1 = await listen('file-changed', debouncedRefresh);
    const unlisten2 = await listen('reminder-fired', () => refreshItems());
    unlisteners = [unlisten1, unlisten2];
  });

  onDestroy(() => {
    unlisteners.forEach(fn => fn());
  });
</script>

<KeyboardHandler>
  <div class="app">
    <Sidebar />
    <MainPane />
  </div>
  <CommandPalette />
  <ReminderPicker />
</KeyboardHandler>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }
  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #1e1e1e;
    color: #d4d4d4;
    overflow: hidden;
  }
  .app {
    display: flex;
    height: 100vh;
    width: 100vw;
  }
</style>
