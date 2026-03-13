<script lang="ts">
  import { openTabs, activeTabIndex } from './stores';
  import { switchTab as doSwitchTab, closeTabByIndex } from './actions';

  function closeTab(index: number, e: MouseEvent) {
    e.stopPropagation();
    closeTabByIndex(index);
  }
</script>

{#if $openTabs.length > 0}
  <div class="tab-bar" role="tablist">
    {#each $openTabs as tab, i}
      <div
        class="tab"
        class:active={$activeTabIndex === i}
        on:click={() => { if (i !== $activeTabIndex) doSwitchTab(i); }}
        on:keydown={(e) => { if (e.key === 'Enter' && i !== $activeTabIndex) doSwitchTab(i); }}
        role="tab"
        tabindex="0"
      >
        <span class="tab-name">{tab.filename}</span>
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
</style>
