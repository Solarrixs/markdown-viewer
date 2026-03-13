<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import KeyboardHandler from './lib/KeyboardHandler.svelte';
  import Sidebar from './lib/Sidebar.svelte';
  import MainPane from './lib/MainPane.svelte';
  import CommandPalette from './lib/CommandPalette.svelte';
  import ReminderPicker from './lib/ReminderPicker.svelte';
  import SettingsModal from './lib/SettingsModal.svelte';
  import { refreshItems, saveIfDirty, closeActiveTab, openFileDialog } from './lib/actions';
  import { settingsOpen, commandPaletteOpen, editMode, showDiff, sidebarVisible, selfSaveInFlight } from './lib/stores';
  import { get } from 'svelte/store';

  let unlisteners: Array<() => void> = [];
  let debounceTimer: ReturnType<typeof setTimeout>;

  function debouncedRefresh() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      // Check at execution time, not arrival time, to avoid race with the 500ms flag window
      if (!get(selfSaveInFlight)) refreshItems();
    }, 300);
  }

  onMount(async () => {
    await refreshItems();

    const unlisten1 = await listen('file-changed', debouncedRefresh);
    const unlisten2 = await listen('reminder-fired', () => refreshItems());

    // Menu bar event handlers
    const menuListeners = await Promise.all([
      listen('settings', () => settingsOpen.set(true)),
      listen('save', () => saveIfDirty()),
      listen('close_tab', () => closeActiveTab()),
      listen('edit_mode', async () => {
        await saveIfDirty();
        editMode.update(v => !v);
        showDiff.set(false);
      }),
      listen('diff_view', () => showDiff.update(v => !v)),
      listen('toggle_sidebar', () => sidebarVisible.update(v => !v)),
      listen('command_palette', () => commandPaletteOpen.set(true)),
      listen('open_file', () => openFileDialog()),
    ]);

    unlisteners = [unlisten1, unlisten2, ...menuListeners];
  });

  onDestroy(() => {
    clearTimeout(debounceTimer);
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
  <SettingsModal />
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
    color: #eee;
    overflow: hidden;
  }
  .app {
    display: flex;
    height: 100vh;
    width: 100vw;
  }
</style>
