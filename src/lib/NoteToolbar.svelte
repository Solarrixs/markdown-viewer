<script lang="ts">
  import { toolbarFeatures, featureContext, executeFeature } from './registry';
  import type { ToolbarEntry, ResolvedFeature } from './registry';
  import { activeFilePath, tick } from './stores';
  import { timeAgo, timeUntil } from './utils';

  $: currentItem = $featureContext.currentItem;

  function isFeature(entry: ToolbarEntry): entry is ResolvedFeature {
    return !('separator' in entry);
  }
</script>

{#if $activeFilePath}
  <div class="toolbar">
    <div class="toolbar-left">
      {#each $toolbarFeatures as entry}
        {#if isFeature(entry)}
          <button class="tool-btn" class:active={entry.isActive} on:click={() => executeFeature(entry, $featureContext)}>
            {entry.icon}
            <span class="btn-label">{entry.resolvedLabel}</span>
            {#if entry.shortcutHint}
              <kbd>{entry.shortcutHint}</kbd>
            {/if}
          </button>
        {:else}
          <span class="separator"></span>
        {/if}
      {/each}
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
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
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
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
    border-radius: 3px;
    white-space: nowrap;
  }
  .tool-btn:hover {
    background: var(--bg-hover);
    color: var(--text-heading);
  }
  .tool-btn.active {
    color: var(--accent);
  }
  .btn-label {
    font-size: 10px;
  }
  .tool-btn :global(kbd) {
    font-size: 9px;
    font-family: monospace;
    color: var(--text-disabled);
    background: var(--bg-active);
    border: 1px solid var(--border);
    border-radius: 2px;
    padding: 0 3px;
    margin-left: 2px;
  }
  .separator {
    width: 1px;
    height: 16px;
    background: var(--border);
    margin: 0 4px;
  }
  .reminder-badge {
    font-size: 10px;
    color: var(--accent);
    margin-right: 12px;
  }
  .last-edited {
    font-size: 10px;
    color: var(--text-secondary);
  }
</style>
