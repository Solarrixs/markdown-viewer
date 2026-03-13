<script lang="ts">
  import type { InboxItem } from './stores';

  export let item: InboxItem;
  export let selected: boolean = false;

  function timeAgo(dateStr: string | null): string {
    if (!dateStr) return '';
    const diff = Date.now() - new Date(dateStr).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'now';
    if (mins < 60) return `${mins}m`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h`;
    const days = Math.floor(hours / 24);
    return `${days}d`;
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
      {#if item.pinned}<span class="pin-icon">&#x1F4CC;</span>{/if}
      {#if item.reminder_time}<span class="clock-icon">&#x23F0;</span>{/if}
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
    <span class="time">{timeAgo(item.last_modified)}</span>
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
  .file-item:hover { background: #1e1e1e; }
  .file-item.selected {
    background: #1e1e2e;
    border-left-color: #5b9bd5;
  }
  .file-item.unread .filename { color: #e0e0e0; font-weight: 500; }
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
    color: #999;
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
  .time { color: #555; font-size: 11px; }
</style>
