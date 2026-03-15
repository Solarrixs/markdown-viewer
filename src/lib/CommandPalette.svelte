<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { get } from 'svelte/store';
  import { commandPaletteOpen } from './stores';
  import { openFile, openFilePath } from './actions';
  import { paletteFeatures, featureContext, executeFeature } from './registry';
  import { invoke } from '@tauri-apps/api/core';

  let query = '';
  let selectedResultIndex = 0;
  let inputEl: HTMLInputElement;
  let searchDebounceTimer: ReturnType<typeof setTimeout>;

  interface PaletteItem {
    label: string;
    hint: string;
    type: 'action' | 'file' | 'path' | 'content';
    action: () => void;
  }

  interface PaletteSection {
    label: string;
    items: PaletteItem[];
  }

  let fileResults: PaletteItem[] = [];
  let contentResults: PaletteItem[] = [];

  $: if ($commandPaletteOpen) {
    query = '';
    selectedResultIndex = 0;
    fileResults = [];
    contentResults = [];
    setTimeout(() => inputEl?.focus(), 50);
  }

  $: filteredActions = query
    ? $paletteFeatures.filter(a => a.resolvedLabel.toLowerCase().includes(query.toLowerCase()))
    : $paletteFeatures;

  $: {
    clearTimeout(searchDebounceTimer);
    if (isPathQuery) {
      fileResults = [];
      contentResults = [];
    } else if (query.length >= 2) {
      searchDebounceTimer = setTimeout(() => { searchFiles(query); searchContent(query); }, 150);
    } else {
      fileResults = [];
      contentResults = [];
    }
  }

  $: isPathQuery = query.startsWith('/') || query.startsWith('~');

  $: sections = ((): PaletteSection[] => {
    const result: PaletteSection[] = [];
    if (isPathQuery) {
      result.push({ label: 'Open Path', items: [{
        label: `Open ${query}`, hint: '', type: 'path',
        action: () => { close(); openFilePath(query); },
      }] });
    } else if (filteredActions.length > 0) {
      const actionItems: PaletteItem[] = filteredActions.map(f => ({
        label: f.resolvedLabel,
        hint: f.shortcutHint ?? '',
        type: 'action' as const,
        action: () => { executeFeature(f, get(featureContext)); close(); },
      }));
      result.push({ label: 'Actions', items: actionItems });
    }
    if (fileResults.length > 0) {
      result.push({ label: 'Files', items: fileResults });
    }
    if (contentResults.length > 0) {
      result.push({ label: 'Content', items: contentResults });
    }
    return result;
  })();

  $: allResults = sections.flatMap(s => s.items);
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

  async function searchContent(q: string) {
    try {
      const results = await invoke<{ path: string; filename: string; line_number: number; context: string }[]>('search_file_contents', { query: q });
      contentResults = results.map(r => ({
        label: r.filename,
        hint: r.context,
        type: 'content' as const,
        action: () => {
          close();
          openFile({ path: r.path, filename: r.filename });
        },
      }));
    } catch {
      contentResults = [];
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
  <div class="backdrop" transition:fade={{ duration: 150 }} on:click={close} on:keydown={(e) => { if (e.key === 'Escape') close(); }} role="button" tabindex="-1">
    <div class="palette" transition:scale={{ start: 0.98, duration: 150 }} on:click|stopPropagation role="dialog" tabindex="-1" on:keydown={() => {}}>
      <input
        bind:this={inputEl}
        bind:value={query}
        on:keydown={handleKeydown}
        placeholder="Type a command, search files, or paste a path..."
        class="search-input"
      />
      <div class="results">
        {#each sections as section, si}
          <div class="section-label">{section.label}</div>
          {#each section.items as item, i}
            {@const idx = sections.slice(0, si).reduce((sum, s) => sum + s.items.length, 0) + i}
            <button
              class="result"
              class:selected={selectedResultIndex === idx}
              on:click={() => item.action()}
              on:mouseenter={() => selectedResultIndex = idx}
            >
              <span class="result-label">{item.label}</span>
              <span class="result-hint" class:file-hint={item.type === 'file'} class:content-hint={item.type === 'content'}>{item.hint}</span>
            </button>
          {/each}
        {/each}
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
    background: var(--bg-elevated);
    border: 1px solid var(--border);
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
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
    font-size: 14px;
    outline: none;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .search-input::placeholder { color: var(--text-disabled); }
  .results {
    overflow-y: auto;
    max-height: 340px;
    padding: 4px;
  }
  .section-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-disabled);
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
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    border-radius: 4px;
    text-align: left;
  }
  .result:hover, .result.selected { background: var(--bg-hover); }
  .result-hint { color: var(--text-disabled); font-size: 11px; font-family: monospace; }
  .file-hint { max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .content-hint {
    max-width: 250px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
    color: var(--text-disabled);
  }
  .no-results { padding: 16px; text-align: center; color: var(--text-disabled); font-size: 13px; }
</style>
