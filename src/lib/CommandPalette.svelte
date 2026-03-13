<script lang="ts">
  import { commandPaletteOpen, inboxItems, activeFilePath, settingsOpen } from './stores';
  import { switchSection, toggleAlwaysOnTop, archiveFile, togglePin, openFile } from './actions';
  import { invoke } from '@tauri-apps/api/core';

  let query = '';
  let selectedResultIndex = 0;
  let inputEl: HTMLInputElement;
  let searchDebounceTimer: ReturnType<typeof setTimeout>;

  interface PaletteItem {
    label: string;
    hint: string;
    type: 'action' | 'file';
    action: () => void;
  }

  const actions: PaletteItem[] = [
    { label: 'Archive file', hint: 'E', type: 'action', action: () => { archiveFile(); close(); } },
    { label: 'Pin / Unpin file', hint: 'P', type: 'action', action: () => { togglePin(); close(); } },
    { label: 'Set reminder', hint: 'H', type: 'action', action: () => {} },
    { label: 'Toggle edit mode', hint: 'Cmd+E', type: 'action', action: () => {} },
    { label: 'Toggle always on top', hint: '', type: 'action', action: () => { toggleAlwaysOnTop(); close(); } },
    { label: 'Go to Inbox', hint: 'G I', type: 'action', action: () => { switchSection('inbox'); close(); } },
    { label: 'Go to Pinned', hint: 'G P', type: 'action', action: () => { switchSection('pinned'); close(); } },
    { label: 'Go to Reminders', hint: 'G R', type: 'action', action: () => { switchSection('reminders'); close(); } },
    { label: 'Go to Archive', hint: 'G A', type: 'action', action: () => { switchSection('archive'); close(); } },
    { label: 'Open Settings', hint: 'Cmd+,', type: 'action', action: () => { settingsOpen.set(true); close(); } },
  ];

  let fileResults: PaletteItem[] = [];

  $: if ($commandPaletteOpen) {
    query = '';
    selectedResultIndex = 0;
    fileResults = [];
    setTimeout(() => inputEl?.focus(), 50);
  }

  $: filteredActions = query
    ? actions.filter(a => a.label.toLowerCase().includes(query.toLowerCase()))
    : actions;

  $: {
    clearTimeout(searchDebounceTimer);
    if (query.length >= 2) {
      searchDebounceTimer = setTimeout(() => searchFiles(query), 150);
    } else {
      fileResults = [];
    }
  }

  $: allResults = [...filteredActions, ...fileResults];
  $: if (selectedResultIndex >= allResults.length) {
    selectedResultIndex = Math.max(0, allResults.length - 1);
  }

  async function searchFiles(q: string) {
    try {
      const results = await invoke<{ path: string; filename: string }[]>('search_files', { query: q });
      fileResults = results.map(r => ({
        label: r.filename,
        hint: r.path,
        type: 'file' as const,
        action: () => openSearchResult(r),
      }));
    } catch {
      fileResults = [];
    }
  }

  async function openSearchResult(result: { path: string; filename: string }) {
    close();
    await openFile({ path: result.path, filename: result.filename });
  }

  function close() {
    clearTimeout(searchDebounceTimer);
    commandPaletteOpen.set(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedResultIndex = Math.min(selectedResultIndex + 1, allResults.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedResultIndex = Math.max(selectedResultIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = allResults[selectedResultIndex];
      if (item) item.action();
    }
  }
</script>

{#if $commandPaletteOpen}
  <div class="backdrop" on:click={close} on:keydown={(e) => { if (e.key === 'Escape') close(); }} role="button" tabindex="-1">
    <div class="palette" on:click|stopPropagation role="dialog" tabindex="-1" on:keydown={() => {}}>
      <input
        bind:this={inputEl}
        bind:value={query}
        on:keydown={handleKeydown}
        placeholder="Type a command or search files..."
        class="search-input"
      />
      <div class="results">
        {#if filteredActions.length > 0}
          <div class="section-label">Actions</div>
          {#each filteredActions as item, i}
            <button
              class="result"
              class:selected={selectedResultIndex === i}
              on:click={() => item.action()}
              on:mouseenter={() => selectedResultIndex = i}
            >
              <span class="result-label">{item.label}</span>
              <span class="result-hint">{item.hint}</span>
            </button>
          {/each}
        {/if}
        {#if fileResults.length > 0}
          <div class="section-label">Files</div>
          {#each fileResults as item, i}
            {@const idx = filteredActions.length + i}
            <button
              class="result"
              class:selected={selectedResultIndex === idx}
              on:click={() => item.action()}
              on:mouseenter={() => selectedResultIndex = idx}
            >
              <span class="result-label">{item.label}</span>
              <span class="result-hint file-hint">{item.hint}</span>
            </button>
          {/each}
        {/if}
        {#if allResults.length === 0}
          <div class="no-results">No results</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    padding-top: 120px;
    z-index: 100;
  }
  .palette {
    width: 500px;
    max-height: 400px;
    background: #252525;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    height: fit-content;
  }
  .search-input {
    width: 100%;
    padding: 12px 16px;
    background: transparent;
    border: none;
    border-bottom: 1px solid #3a3a3a;
    color: #e0e0e0;
    font-size: 14px;
    outline: none;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .search-input::placeholder { color: #555; }
  .results {
    overflow-y: auto;
    max-height: 340px;
    padding: 4px;
  }
  .section-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #666;
    padding: 8px 12px 4px;
  }
  .result {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
    border-radius: 4px;
    text-align: left;
  }
  .result:hover, .result.selected { background: #333; }
  .result-hint { color: #666; font-size: 11px; font-family: monospace; }
  .file-hint { max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .no-results { padding: 16px; text-align: center; color: #555; font-size: 13px; }
</style>
