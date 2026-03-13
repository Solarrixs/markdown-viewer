<script lang="ts">
  import { openTabs, activeTabIndex, renameTrigger } from './stores';
  import { switchTab as doSwitchTab, closeTabByIndex, renameFile } from './actions';

  let editingIndex: number | null = null;
  let editValue = '';
  let inputEl: HTMLInputElement;
  let renameCommitting = false;
  let lastSeenTrigger = 0;

  // React to rename trigger from keyboard shortcut / command palette
  // Track last-seen value to avoid re-firing on tab switch or mount
  $: if ($renameTrigger > lastSeenTrigger) {
    lastSeenTrigger = $renameTrigger;
    startRename($activeTabIndex);
  }

  function closeTab(index: number, e: MouseEvent) {
    e.stopPropagation();
    closeTabByIndex(index);
  }

  function startRename(index: number) {
    const tab = $openTabs[index];
    if (!tab) return;
    editingIndex = index;
    editValue = tab.filename;
    // Wait for Svelte to render the input, then focus+select
    setTimeout(() => {
      if (inputEl) {
        inputEl.focus();
        inputEl.select();
      }
    }, 0);
  }

  async function commitRename() {
    if (renameCommitting || editingIndex === null) return;
    renameCommitting = true;
    const idx = editingIndex;
    const tab = $openTabs[idx];
    const newName = editValue.trim();
    editingIndex = null;
    if (tab && newName && newName !== tab.filename) {
      await renameFile(tab.path, newName);
    }
    renameCommitting = false;
  }

  function cancelRename() {
    editingIndex = null;
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      commitRename();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelRename();
    }
  }
</script>

{#if $openTabs.length > 0}
  <div class="tab-bar" role="tablist">
    {#each $openTabs as tab, i}
      <div
        class="tab"
        class:active={$activeTabIndex === i}
        on:click={() => { if (editingIndex === null && i !== $activeTabIndex) doSwitchTab(i); }}
        on:dblclick|stopPropagation={() => startRename(i)}
        on:keydown={(e) => { if (e.key === 'Enter' && i !== $activeTabIndex) doSwitchTab(i); }}
        role="tab"
        tabindex="0"
      >
        {#if editingIndex === i}
          <input
            bind:this={inputEl}
            bind:value={editValue}
            on:blur={commitRename}
            on:keydown={handleRenameKeydown}
            on:click|stopPropagation
            class="rename-input"
            spellcheck="false"
          />
        {:else}
          <span class="tab-name">{tab.filename}</span>
        {/if}
        {#if tab.additions > 0 || tab.deletions > 0}
          <span class="tab-diff">
            <span class="add">+{tab.additions}</span>
            <span class="del">-{tab.deletions}</span>
          </span>
        {/if}
        <button class="close" on:click={(e) => closeTab(i, e)}>&times;</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .tab-bar {
    display: flex;
    background: var(--bg-base);
    border-bottom: 1px solid var(--border);
    overflow-x: auto;
    flex-shrink: 0;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-right: 1px solid var(--border);
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: all var(--duration-fast) ease;
  }
  .tab:hover { color: var(--text-primary); }
  .tab.active {
    color: var(--text-heading);
    background: var(--bg-surface);
    border-bottom: 2px solid var(--accent);
  }
  .tab-diff { font-family: monospace; font-size: 10px; }
  .add { color: #4ec9b0; }
  .del { color: #d16969; }
  .close {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 14px;
    cursor: pointer;
    padding: 0 2px;
    line-height: 1;
  }
  .close:hover { color: var(--text-primary); }
  .rename-input {
    background: var(--bg-base);
    border: 1px solid var(--accent);
    border-radius: 3px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: inherit;
    padding: 1px 4px;
    outline: none;
    width: 120px;
  }
</style>
