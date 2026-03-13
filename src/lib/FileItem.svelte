<script lang="ts">
  import type { InboxItem } from './stores';
  import { tick } from './stores';
  import { timeAgo } from './utils';

  import { createEventDispatcher } from 'svelte';

  export let item: InboxItem;
  export let selected: boolean = false;

  const dispatch = createEventDispatcher<{ dismiss: void }>();

  function handleDismiss(e: MouseEvent) {
    e.stopPropagation();
    dispatch('dismiss');
  }
</script>

<div class="file-item" class:selected class:unread={item.status === 'unread'}>
  <div class="left">
    {#if item.status === 'unread'}
      <span class="dot"></span>
    {:else}
      <span class="dot-placeholder"></span>
    {/if}
    <span class="filename" title={item.path}>
      {#if item.pinned}<span class="pin-icon">📌</span>{/if}
      {#if item.reminder_time}<span class="clock-icon">⏰</span>{/if}
      {item.filename}
    </span>
  </div>
  <div class="right">
    {#if item.additions > 0 || item.deletions > 0}
      <span class="diff-badge">
        <span class="add">+{item.additions}</span>
        <span class="del">-{item.deletions}</span>
      </span>
    {/if}
    <span class="time">{void $tick, timeAgo(item.last_modified)}</span>
    <button class="dismiss" on:click={handleDismiss} title="Archive">&times;</button>
  </div>
</div>

<style>
  .file-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    cursor: pointer;
    transition: background 0.1s;
    font-size: 13px;
    border-left: 2px solid transparent;
  }
  .file-item:hover { background: #252525; }
  .file-item.selected {
    background: #222236;
    border-left-color: #5b9bd5;
  }
  .file-item.unread .filename { color: #fff; font-weight: 500; }
  .left {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex: 1;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #5b9bd5;
    flex-shrink: 0;
  }
  .dot-placeholder { width: 6px; flex-shrink: 0; }
  .filename {
    color: #ccc;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pin-icon, .clock-icon { font-size: 10px; margin-right: 2px; }
  .right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .diff-badge { font-size: 10px; font-family: monospace; }
  .add { color: #4ec9b0; }
  .del { color: #d16969; }
  .time { color: #999; font-size: 11px; }
  .dismiss {
    display: none;
    background: transparent;
    border: none;
    color: #666;
    font-size: 14px;
    cursor: pointer;
    padding: 0 2px;
    line-height: 1;
    flex-shrink: 0;
  }
  .dismiss:hover { color: #e0e0e0; }
  .file-item:hover .dismiss { display: block; }
</style>
