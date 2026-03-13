<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { shortcutHelpOpen } from './stores';

  let query = '';
  let inputEl: HTMLInputElement;

  interface Shortcut {
    keys: string;
    label: string;
    category: string;
  }

  const shortcuts: Shortcut[] = [
    // Navigation
    { keys: 'J', label: 'Next file', category: 'Navigation' },
    { keys: 'K', label: 'Previous file', category: 'Navigation' },
    { keys: 'Enter', label: 'Open selected file', category: 'Navigation' },
    { keys: 'G I', label: 'Go to Inbox', category: 'Navigation' },
    { keys: 'G P', label: 'Go to Pinned', category: 'Navigation' },
    { keys: 'G R', label: 'Go to Reminders', category: 'Navigation' },
    { keys: 'G A', label: 'Go to Archive', category: 'Navigation' },
    // Actions
    { keys: 'E', label: 'Archive file', category: 'Actions' },
    { keys: 'P', label: 'Pin / Unpin file', category: 'Actions' },
    { keys: 'H', label: 'Set reminder', category: 'Actions' },
    { keys: 'Z', label: 'Undo last action', category: 'Actions' },
    // View
    { keys: '\u2318+E', label: 'Toggle edit mode', category: 'View' },
    { keys: '\u2318+D', label: 'Toggle diff view', category: 'View' },
    { keys: '\u2318+\\', label: 'Toggle sidebar', category: 'View' },
    { keys: 'O', label: 'Toggle table of contents', category: 'View' },
    { keys: '\u2318+F', label: 'Find in file', category: 'View' },
    { keys: '\u2318+Enter', label: 'Open in split view', category: 'View' },
    // Tabs
    { keys: '\u2318+W', label: 'Close tab', category: 'Tabs' },
    { keys: '\u2318+Shift+T', label: 'Reopen closed tab', category: 'Tabs' },
    { keys: '\u2318+1-9', label: 'Switch to tab', category: 'Tabs' },
    // Global
    { keys: '\u2318+K', label: 'Command palette', category: 'Global' },
    { keys: '\u2318+O', label: 'Open file', category: 'Global' },
    { keys: '\u2318+,', label: 'Settings', category: 'Global' },
    { keys: 'F', label: 'Reveal in Finder', category: 'Global' },
    { keys: 'T', label: 'Open in Terminal', category: 'Global' },
    { keys: 'C', label: 'Copy file path', category: 'Global' },
    { keys: '?', label: 'Show this help', category: 'Global' },
  ];

  $: if ($shortcutHelpOpen) {
    query = '';
    setTimeout(() => inputEl?.focus(), 50);
  }

  $: filtered = query
    ? shortcuts.filter(s =>
        s.label.toLowerCase().includes(query.toLowerCase()) ||
        s.keys.toLowerCase().includes(query.toLowerCase()) ||
        s.category.toLowerCase().includes(query.toLowerCase())
      )
    : shortcuts;

  $: categories = [...new Set(filtered.map(s => s.category))];

  function close() {
    shortcutHelpOpen.set(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

{#if $shortcutHelpOpen}
  <div class="backdrop" transition:fade={{ duration: 150 }} on:click={close} on:keydown={handleKeydown} role="button" tabindex="-1">
    <div class="overlay" transition:scale={{ start: 0.98, duration: 150 }} on:click|stopPropagation on:keydown={handleKeydown} role="dialog" tabindex="-1">
      <div class="header">
        <h2>Keyboard Shortcuts</h2>
        <button class="close-btn" on:click={close}>&times;</button>
      </div>
      <div class="search-wrap">
        <input
          bind:this={inputEl}
          bind:value={query}
          on:keydown={handleKeydown}
          placeholder="Search shortcuts..."
          class="search-input"
        />
      </div>
      <div class="shortcuts-body">
        <div class="columns">
          {#each categories as category}
            <div class="category">
              <div class="category-label">{category}</div>
              {#each filtered.filter(s => s.category === category) as shortcut}
                <div class="shortcut-row">
                  <span class="shortcut-label">{shortcut.label}</span>
                  <span class="shortcut-keys">
                    {#each shortcut.keys.split('+') as part, i}
                      {#if i > 0}<span class="plus">+</span>{/if}
                      <kbd>{part.trim()}</kbd>
                    {/each}
                  </span>
                </div>
              {/each}
            </div>
          {/each}
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
    padding-top: 60px;
    z-index: 100;
  }
  .overlay {
    width: 600px;
    max-height: 500px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    height: fit-content;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px 12px;
    border-bottom: 1px solid var(--border);
  }
  .header h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-disabled);
    font-size: 20px;
    cursor: pointer;
    padding: 0 4px;
  }
  .close-btn:hover { color: var(--text-secondary); }
  .search-wrap {
    padding: 12px 20px;
  }
  .search-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .search-input:focus { border-color: var(--accent); }
  .search-input::placeholder { color: var(--text-disabled); }
  .shortcuts-body {
    overflow-y: auto;
    padding: 0 20px 16px;
    max-height: 380px;
  }
  .columns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .category {
    min-width: 0;
  }
  .category-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-disabled);
    margin-bottom: 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .shortcut-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 0;
    gap: 8px;
  }
  .shortcut-label {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .shortcut-keys {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }
  .shortcut-keys :global(kbd) {
    display: inline-block;
    background: var(--bg-active);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 5px;
    font-family: monospace;
    font-size: 11px;
    color: var(--text-secondary);
    min-width: 20px;
    text-align: center;
  }
  .plus {
    font-size: 10px;
    color: var(--text-disabled);
    margin: 0 1px;
  }
</style>
