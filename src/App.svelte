<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import KeyboardHandler from './lib/KeyboardHandler.svelte';
  import Sidebar from './lib/Sidebar.svelte';
  import MainPane from './lib/MainPane.svelte';
  import CommandPalette from './lib/CommandPalette.svelte';
  import ReminderPicker from './lib/ReminderPicker.svelte';
  import SettingsModal from './lib/SettingsModal.svelte';
  import ShortcutHelp from './lib/ShortcutHelp.svelte';
  import BulkOpenModal from './lib/BulkOpenModal.svelte';
  import Toast from './lib/Toast.svelte';
  import { refreshItems, saveIfDirty, closeActiveTab, openFileDialog, loadRecentCommits } from './lib/actions';
  import { settingsOpen, commandPaletteOpen, editMode, showDiff, sidebarVisible, selfSaveInFlight, activeFilePath, openTabs, sidebarViewMode } from './lib/stores';
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
    const unlisten4 = await listen('new-commits', () => {
      // Auto-refresh timeline if it's visible
      if (get(sidebarViewMode) === 'timeline') {
        loadRecentCommits();
      }
    });
    const unlisten3 = await listen<string>('file-removed', (event) => {
      const removedPath = event.payload;
      // Close tab if the removed file is open
      openTabs.update(tabs => {
        const idx = tabs.findIndex(t => t.path === removedPath);
        if (idx !== -1) {
          const newTabs = [...tabs];
          newTabs.splice(idx, 1);
          return newTabs;
        }
        return tabs;
      });
      // Clear active file if it was removed
      activeFilePath.update(p => p === removedPath ? null : p);
      refreshItems();
    });

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

    unlisteners = [unlisten1, unlisten2, unlisten3, unlisten4, ...menuListeners];
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
  <ShortcutHelp />
  <BulkOpenModal />
  <Toast />
</KeyboardHandler>

<style>
  :global(:root) {
    --bg-base: #161616;
    --bg-surface: #1c1c1c;
    --bg-elevated: #1e1e1e;
    --bg-overlay: #1e1e1e;
    --bg-hover: #242424;
    --bg-active: #2a2a2a;
    --border: #2a2a2a;
    --border-subtle: #222;
    --text-primary: rgba(255, 255, 255, 0.87);
    --text-heading: #f0f0f0;
    --text-secondary: rgba(255, 255, 255, 0.6);
    --text-disabled: rgba(255, 255, 255, 0.38);
    --accent: #5b9bd5;
    --ease-out: cubic-bezier(0.16, 1, 0.3, 1);
    --ease-in: cubic-bezier(0.7, 0, 0.84, 0);
    --duration-fast: 100ms;
    --duration-normal: 150ms;
    --duration-slow: 200ms;
  }
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }
  :global(body) {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: var(--bg-base);
    color: var(--text-primary);
    overflow: hidden;
  }
  .app {
    display: flex;
    height: 100vh;
    width: 100vw;
  }
</style>
