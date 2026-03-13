<script lang="ts">
  import { activeFilePath, inboxItems, reminderPickerOpen } from './stores';
  import { togglePin, openInFinder, openInVSCode, openInTerminal, copyPath } from './actions';
  import { timeAgo } from './utils';

  $: currentItem = $inboxItems.find(i => i.path === $activeFilePath);
</script>

{#if $activeFilePath}
  <div class="toolbar">
    <div class="toolbar-left">
      <button class="tool-btn" class:active={currentItem?.pinned} on:click={togglePin} title={currentItem?.pinned ? 'Unpin' : 'Pin'}>
        📌
        <span class="btn-label">{currentItem?.pinned ? 'Unpin' : 'Pin'}</span>
      </button>
      <button class="tool-btn" on:click={() => reminderPickerOpen.set(true)} title="Set Reminder">
        ⏰ <span class="btn-label">Remind</span>
      </button>
      <span class="separator"></span>
      <button class="tool-btn" on:click={openInFinder} title="Reveal in Finder">
        📁 <span class="btn-label">Finder</span>
      </button>
      <button class="tool-btn" on:click={openInVSCode} title="Open in VS Code">
        ⌨ <span class="btn-label">VS Code</span>
      </button>
      <button class="tool-btn" on:click={openInTerminal} title="Open in Terminal">
        ▶ <span class="btn-label">Terminal</span>
      </button>
      <button class="tool-btn" on:click={copyPath} title="Copy File Path">
        📋 <span class="btn-label">Copy Path</span>
      </button>
    </div>
    <div class="toolbar-right">
      {#if currentItem?.last_modified}
        <span class="last-edited">Edited {timeAgo(currentItem.last_modified, true)}</span>
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
  .separator {
    width: 1px;
    height: 16px;
    background: #444;
    margin: 0 4px;
  }
  .last-edited {
    font-size: 10px;
    color: #999;
  }
</style>
