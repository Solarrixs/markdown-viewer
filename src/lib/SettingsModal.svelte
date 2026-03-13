<script lang="ts">
  import { settingsOpen } from './stores';
  import type { WatchedFolder, IgnorePattern } from './stores';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';

  let folders: WatchedFolder[] = [];
  let patterns: IgnorePattern[] = [];
  let newPattern = '';

  $: if ($settingsOpen) {
    loadSettings();
  }

  async function loadSettings() {
    try {
      folders = await invoke<WatchedFolder[]>('get_watched_folders');
      patterns = await invoke<IgnorePattern[]>('get_ignore_patterns');
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  async function mutate(command: string, args: Record<string, unknown>) {
    try {
      await invoke(command, args);
      await loadSettings();
      // No refreshItems() — watcher restart handles re-scanning
    } catch (e) {
      console.error(`Failed to ${command}:`, e);
    }
  }

  async function addFolder() {
    const selected = await open({ directory: true, multiple: false }).catch(() => null);
    if (selected) await mutate('add_watched_folder', { path: selected });
  }

  async function removeFolder(id: number) {
    await mutate('remove_watched_folder', { id });
  }

  async function addPattern() {
    if (!newPattern.trim()) return;
    const pattern = newPattern.trim();
    newPattern = '';
    await mutate('add_ignore_pattern', { pattern });
  }

  async function removePattern(id: number) {
    await mutate('remove_ignore_pattern', { id });
  }

  function close() {
    settingsOpen.set(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
    if (e.key === 'Enter' && newPattern.trim()) {
      e.preventDefault();
      addPattern();
    }
  }
</script>

{#if $settingsOpen}
  <div class="backdrop" on:click={close} on:keydown={(e) => { if (e.key === 'Escape') close(); }} role="button" tabindex="-1">
    <div class="modal" on:click|stopPropagation on:keydown={handleKeydown} role="dialog" tabindex="-1">
      <div class="modal-header">
        <h2>Settings</h2>
        <button class="close-btn" on:click={close}>&times;</button>
      </div>

      <div class="section">
        <h3>Watched Folders</h3>
        <div class="list">
          {#each folders as folder}
            <div class="list-item">
              <span class="item-text" title={folder.path}>{folder.path}</span>
              <button class="remove-btn" on:click={() => removeFolder(folder.id)}>&times;</button>
            </div>
          {/each}
        </div>
        <button class="add-btn" on:click={addFolder}>+ Add Folder</button>
      </div>

      <div class="section">
        <h3>Exclusion Patterns</h3>
        <div class="list">
          {#each patterns as pattern}
            <div class="list-item">
              <span class="item-text"><code>{pattern.pattern}</code></span>
              <button class="remove-btn" on:click={() => removePattern(pattern.id)}>&times;</button>
            </div>
          {/each}
        </div>
        <div class="add-pattern">
          <input
            bind:value={newPattern}
            placeholder="e.g. *.tmp, .hidden/*"
            class="pattern-input"
          />
          <button class="add-btn" on:click={addPattern}>Add</button>
        </div>
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
    padding-top: 80px;
    z-index: 100;
  }
  .modal {
    width: 520px;
    max-height: 500px;
    background: #252525;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    height: fit-content;
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px 12px;
    border-bottom: 1px solid #3a3a3a;
  }
  .modal-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: #e0e0e0;
    margin: 0;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: #666;
    font-size: 20px;
    cursor: pointer;
    padding: 0 4px;
  }
  .close-btn:hover { color: #ccc; }
  .section {
    padding: 16px 20px;
  }
  .section + .section {
    border-top: 1px solid #2a2a2a;
  }
  .section h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #888;
    margin: 0 0 8px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 8px;
    max-height: 160px;
    overflow-y: auto;
  }
  .list-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    background: #1e1e1e;
    border-radius: 4px;
  }
  .item-text {
    font-size: 12px;
    color: #ccc;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }
  .item-text code {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 11px;
    color: #5b9bd5;
  }
  .remove-btn {
    background: transparent;
    border: none;
    color: #555;
    font-size: 16px;
    cursor: pointer;
    padding: 0 4px;
    flex-shrink: 0;
  }
  .remove-btn:hover { color: #d16969; }
  .add-btn {
    padding: 6px 12px;
    background: #2a2a2a;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    color: #ccc;
    font-size: 12px;
    cursor: pointer;
  }
  .add-btn:hover { background: #333; }
  .add-pattern {
    display: flex;
    gap: 8px;
  }
  .pattern-input {
    flex: 1;
    padding: 6px 10px;
    background: #1e1e1e;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    color: #e0e0e0;
    font-size: 12px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    outline: none;
  }
  .pattern-input:focus { border-color: #5b9bd5; }
  .pattern-input::placeholder { color: #555; }
</style>
