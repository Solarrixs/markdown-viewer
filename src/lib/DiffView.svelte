<script lang="ts">
  import { fileDiff, annotations, activeAnnotationLine, selectedCommitOid } from './stores';
  import { loadAnnotations } from './actions';
  import AnnotationPopover from './AnnotationPopover.svelte';
  import DiffSummaryCard from './DiffSummaryCard.svelte';
  import { get } from 'svelte/store';
  import { activeFilePath } from './stores';

  // Load annotations when file changes
  $: if ($activeFilePath) {
    loadAnnotations($activeFilePath);
  }

  $: annotatedLines = new Set(($annotations ?? []).map(a => a.line_number));

  function toggleAnnotation(lineNum: number) {
    if (get(activeAnnotationLine) === lineNum) {
      activeAnnotationLine.set(null);
    } else {
      activeAnnotationLine.set(lineNum);
    }
  }
</script>

<div class="diff-view">
  {#if $selectedCommitOid}
    <DiffSummaryCard commitOid={$selectedCommitOid} />
  {/if}

  {#if $fileDiff && $fileDiff.hunks.length > 0}
    <div class="diff-header">
      <span class="add">+{$fileDiff.additions}</span>
      <span class="del">-{$fileDiff.deletions}</span>
    </div>
    <div class="diff-lines">
      {#each $fileDiff.hunks as hunk, i}
        <div
          class="diff-line {hunk.change_type}"
          on:click={() => toggleAnnotation(hunk.new_start)}
          on:keydown={() => {}}
          role="button"
          tabindex="-1"
        >
          <span class="gutter">
            <span class="line-num old">{hunk.old_start || ''}</span>
            <span class="line-num new">{hunk.new_start || ''}</span>
            {#if annotatedLines.has(hunk.new_start)}
              <span class="annotation-dot" title="Has annotation">●</span>
            {/if}
          </span>
          <span class="line-prefix">{#if hunk.change_type === 'added'}+{:else if hunk.change_type === 'removed'}-{:else}{' '}{/if}</span>
          <span class="line-content">{hunk.content}</span>
        </div>
        {#if $activeAnnotationLine === hunk.new_start}
          <AnnotationPopover
            lineNumber={hunk.new_start}
            filePath={$activeFilePath ?? ''}
            commitHash={$selectedCommitOid}
          />
        {/if}
      {/each}
    </div>
  {:else}
    <div class="no-diff">
      <p>No changes from last commit</p>
    </div>
  {/if}
</div>

<style>
  .diff-view {
    padding: 24px;
  }
  .diff-header {
    margin-bottom: 12px;
    font-family: monospace;
    font-size: 13px;
  }
  .add { color: #4ec9b0; margin-right: 8px; }
  .del { color: #d16969; }
  .diff-lines {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 13px;
    line-height: 1.5;
    background: var(--bg-base);
    border-radius: 6px;
    overflow-x: auto;
  }
  .diff-line {
    display: flex;
    align-items: stretch;
    cursor: pointer;
    min-height: 20px;
  }
  .diff-line:hover {
    filter: brightness(1.1);
  }
  .diff-line.added {
    background: rgba(78, 201, 176, 0.08);
  }
  .diff-line.removed {
    background: rgba(209, 105, 105, 0.08);
  }
  .diff-line.context {
    background: transparent;
  }
  .gutter {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 4px;
    min-width: 90px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-subtle);
    user-select: none;
    -webkit-user-select: none;
  }
  .line-num {
    display: inline-block;
    width: 32px;
    text-align: right;
    font-size: 11px;
    color: var(--text-disabled);
  }
  .annotation-dot {
    color: var(--accent);
    font-size: 10px;
    margin-left: 2px;
  }
  .line-prefix {
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }
  .diff-line.added .line-prefix { color: #4ec9b0; }
  .diff-line.removed .line-prefix { color: #d16969; }
  .diff-line.context .line-prefix { color: var(--text-disabled); }
  .line-content {
    flex: 1;
    white-space: pre;
    padding-right: 16px;
  }
  .diff-line.added .line-content { color: #4ec9b0; }
  .diff-line.removed .line-content { color: #d16969; }
  .diff-line.context .line-content { color: var(--text-disabled); }
  .no-diff {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-disabled);
    font-size: 13px;
  }
</style>
