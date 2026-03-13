<script lang="ts">
  import { slide, fade } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { sectionItems, selectedIndex } from './stores';
  import type { InboxItem } from './stores';
  import FileItem from './FileItem.svelte';
  import { openFile, archiveByPath } from './actions';

  function handleClick(item: InboxItem, index: number) {
    openFile(item, index);
  }
</script>

<div class="file-list">
  {#if $sectionItems.length === 0}
    <div class="empty">
      <span class="check">✓</span>
      <p>All caught up</p>
    </div>
  {:else}
    {#each $sectionItems as item, i (item.path)}
      <button class="file-button" animate:flip={{ duration: 200 }} transition:slide|local={{ duration: 200 }} on:click={() => handleClick(item, i)}>
        <FileItem {item} selected={$selectedIndex === i} on:dismiss={() => archiveByPath(item.path)} />
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
    color: var(--text-secondary);
  }
  .check { font-size: 32px; color: #4ec9b0; }
  .empty p { margin-top: 8px; font-size: 13px; }
</style>
