<script lang="ts">
  import { fileDiff } from './stores';
</script>

<div class="diff-view">
  {#if $fileDiff && $fileDiff.hunks.length > 0}
    <div class="diff-header">
      <span class="add">+{$fileDiff.additions}</span>
      <span class="del">-{$fileDiff.deletions}</span>
    </div>
    <pre class="diff-content">{#each $fileDiff.hunks as hunk}<span class={hunk.change_type}>{#if hunk.change_type === 'added'}+{:else if hunk.change_type === 'removed'}-{:else}{' '}{/if}{hunk.content}</span>{/each}</pre>
  {:else}
    <div class="no-diff">
      <p>No changes from last commit</p>
    </div>
  {/if}
</div>

<style>
  .diff-view {
    max-width: 720px;
    margin: 0 auto;
    padding: 24px;
  }
  .diff-header {
    margin-bottom: 16px;
    font-family: monospace;
    font-size: 13px;
  }
  .add { color: #4ec9b0; margin-right: 8px; }
  .del { color: #d16969; }
  .diff-content {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 13px;
    line-height: 1.5;
    background: #1a1a1a;
    border-radius: 6px;
    padding: 16px;
    overflow-x: auto;
    white-space: pre;
    margin: 0;
  }
  .diff-content :global(.added) { color: #4ec9b0; background: rgba(78, 201, 176, 0.1); display: inline; }
  .diff-content :global(.removed) { color: #d16969; background: rgba(209, 105, 105, 0.1); display: inline; }
  .diff-content :global(.context) { color: #666; display: inline; }
  .no-diff {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: #555;
    font-size: 13px;
  }
</style>
