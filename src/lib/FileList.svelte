<script lang="ts">
  import { inboxItems, selectedIndex } from './stores';
  import type { InboxItem } from './stores';
  import FileItem from './FileItem.svelte';
  import { openFile } from './actions';

  function handleClick(item: InboxItem, index: number) {
    openFile(item, index);
  }
</script>

<div class="file-list">
  {#if $inboxItems.length === 0}
    <div class="empty">
      <span class="check">✓</span>
      <p>All caught up</p>
    </div>
  {:else}
    {#each $inboxItems as item, i}
      <button class="file-button" on:click={() => handleClick(item, i)}>
        <FileItem {item} selected={$selectedIndex === i} />
      </button>
    {/each}
  {/if}
</div>

<style>
  .file-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .file-button {
    display: block;
    width: 100%;
    padding: 0;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: #999;
  }
  .check { font-size: 32px; color: #4ec9b0; }
  .empty p { margin-top: 8px; font-size: 13px; }
</style>
