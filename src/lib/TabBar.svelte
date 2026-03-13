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
    background: #1a1a1a;
    border-bottom: 1px solid #333;
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
    border-right: 1px solid #333;
    color: #bbb;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.1s;
  }
  .tab:hover { color: #e0e0e0; }
  .tab.active {
    color: #fff;
    background: #1e1e1e;
    border-bottom: 2px solid #5b9bd5;
  }
  .tab-diff { font-family: monospace; font-size: 10px; }
  .add { color: #4ec9b0; }
  .del { color: #d16969; }
  .close {
    background: transparent;
    border: none;
    color: #999;
    font-size: 14px;
    cursor: pointer;
    padding: 0 2px;
    line-height: 1;
  }
  .close:hover { color: #e0e0e0; }
</style>
