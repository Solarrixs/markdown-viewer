<script lang="ts">
  import { selectedCommitOid, reviewStatuses, reviewProgress } from './stores';
  import { setReviewStatus, loadReviewProgress, loadReviewStatuses } from './actions';
  import { onMount } from 'svelte';

  onMount(() => {
    loadReviewProgress();
    loadReviewStatuses();
  });

  $: currentStatus = ($reviewStatuses ?? []).find(r => r.commit_hash === $selectedCommitOid);
</script>

{#if $selectedCommitOid}
  <div class="review-bar">
    <div class="review-actions">
      <button
        class="review-btn reviewed"
        class:active={currentStatus?.status === 'reviewed'}
        on:click={() => setReviewStatus($selectedCommitOid, 'reviewed')}
        title="Mark as reviewed"
      >✓ Reviewed</button>
      <button
        class="review-btn needs-changes"
        class:active={currentStatus?.status === 'needs_changes'}
        on:click={() => setReviewStatus($selectedCommitOid, 'needs_changes')}
        title="Needs changes"
      >✗ Needs Changes</button>
      <button
        class="review-btn skip"
        class:active={currentStatus?.status === 'skipped'}
        on:click={() => setReviewStatus($selectedCommitOid, 'skipped')}
        title="Skip"
      >→ Skip</button>
    </div>
    <div class="progress">
      {$reviewProgress.reviewed}/{$reviewProgress.total} reviewed
    </div>
  </div>
{/if}

<style>
  .review-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 16px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    gap: 12px;
  }
  .review-actions {
    display: flex;
    gap: 6px;
  }
  .review-btn {
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-active);
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
    font-family: inherit;
    transition: all 100ms;
  }
  .review-btn:hover { background: var(--bg-hover); }
  .review-btn.reviewed.active {
    background: rgba(78, 201, 176, 0.15);
    border-color: #4ec9b0;
    color: #4ec9b0;
  }
  .review-btn.needs-changes.active {
    background: rgba(209, 105, 105, 0.15);
    border-color: #d16969;
    color: #d16969;
  }
  .review-btn.skip.active {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--text-disabled);
    color: var(--text-disabled);
  }
  .progress {
    font-size: 11px;
    color: var(--text-disabled);
    white-space: nowrap;
  }
</style>
