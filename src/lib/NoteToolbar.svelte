<script lang="ts">
  import { activeFilePath, sectionItems, reminderPickerOpen, showToc, tick } from './stores';
  import { togglePin, openInFinder, openInTerminal, copyPath } from './actions';
  import { timeAgo, timeUntil } from './utils';

  $: currentItem = $sectionItems.find(i => i.path === $activeFilePath);
</script>

{#if $activeFilePath}
  <div class="toolbar">
    <div class="toolbar-left">
      <button class="tool-btn" class:active={currentItem?.pinned} on:click={togglePin} title={currentItem?.pinned ? 'Unpin' : 'Pin'}>
        📌
        <span class="btn-label">{currentItem?.pinned ? 'Unpin' : 'Pin'}</span>
        <kbd>P</kbd>
      </button>
      <button class="tool-btn" on:click={() => reminderPickerOpen.set(true)} title="Set Reminder">
        ⏰ <span class="btn-label">Remind</span>
        <kbd>H</kbd>
      </button>
      <span class="separator"></span>
      <button class="tool-btn" on:click={openInFinder} title="Reveal in Finder">
        📁 <span class="btn-label">Finder</span>
        <kbd>F</kbd>
      </button>
      <button class="tool-btn" on:click={openInTerminal} title="Open in Ghostty">
        ▶ <span class="btn-label">Terminal</span>
        <kbd>T</kbd>
      </button>
      <button class="tool-btn" on:click={copyPath} title="Copy File Path">
        📋 <span class="btn-label">Copy Path</span>
        <kbd>C</kbd>
      </button>
      <span class="separator"></span>
      <button class="tool-btn" class:active={$showToc} on:click={() => showToc.update(v => !v)} title="Table of Contents">
        ☰ <span class="btn-label">TOC</span>
        <kbd>O</kbd>
      </button>
    </div>
    <div class="toolbar-right">
      {#if currentItem?.reminder_time}
        <span class="reminder-badge">⏰ {void $tick, timeUntil(currentItem.reminder_time)}</span>
      {/if}
      {#if currentItem?.last_modified}
        <span class="last-edited">Edited {void $tick, timeAgo(currentItem.last_modified, true)}</span>
      {/if}
    </div>
  </div>
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 8px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    flex-shrink: 0;
  }
  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .toolbar-right {
    display: flex;
    align-items: center;
  }
  .tool-btn {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 2px 6px;
    background: transparent;
    border: none;
    color: #bbb;
    font-size: 11px;
    cursor: pointer;
    border-radius: 3px;
    white-space: nowrap;
  }
  .tool-btn:hover {
    background: #333;
    color: #f0f0f0;
  }
  .tool-btn.active {
    color: #5b9bd5;
  }
  .btn-label {
    font-size: 10px;
  }
  .tool-btn :global(kbd) {
    font-size: 9px;
    font-family: monospace;
    color: #666;
    background: #2a2a2a;
    border: 1px solid #3a3a3a;
    border-radius: 2px;
    padding: 0 3px;
    margin-left: 2px;
  }
  .separator {
    width: 1px;
    height: 16px;
    background: #444;
    margin: 0 4px;
  }
  .reminder-badge {
    font-size: 10px;
    color: #5b9bd5;
    margin-right: 12px;
  }
  .last-edited {
    font-size: 10px;
    color: #999;
  }
</style>
