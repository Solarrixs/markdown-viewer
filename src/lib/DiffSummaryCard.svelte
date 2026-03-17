<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { DiffSummaryRecord } from './stores';

  export let commitOid: string;

  let summary: string | null = null;
  let loading = false;
  let error: string | null = null;
  let collapsed = false;

  async function loadSummary() {
    try {
      const cached = await invoke<DiffSummaryRecord | null>('get_diff_summary', { commitOid });
      if (cached) {
        summary = cached.summary;
      }
    } catch (_) {}
  }

  async function triggerSummarize() {
    loading = true;
    error = null;
    try {
      summary = await invoke<string>('trigger_summarize', { commitOid });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $: if (commitOid) {
    summary = null;
    error = null;
    loadSummary();
  }
</script>

<div class="summary-card">
  <div class="card-header">
    <button class="collapse-toggle" on:click={() => collapsed = !collapsed}>
      <span class="chevron" class:collapsed>{collapsed ? '▶' : '▼'}</span>
      AI Summary
    </button>
    {#if !summary && !loading}
      <button class="summarize-btn" on:click={triggerSummarize}>Summarize</button>
    {/if}
  </div>
  {#if !collapsed}
    <div class="card-body">
      {#if loading}
        <p class="loading">Generating summary...</p>
      {:else if error}
        <p class="error">{error}</p>
      {:else if summary}
        <p class="summary-text">{summary}</p>
      {:else}
        <p class="placeholder">Click "Summarize" to generate an AI summary of this commit.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .summary-card {
    border: 1px solid var(--border);
    border-radius: 6px;
    margin: 12px 0;
    background: var(--bg-elevated);
    overflow: hidden;
  }
  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--bg-active);
    border-bottom: 1px solid var(--border-subtle);
  }
  .collapse-toggle {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: inherit;
  }
  .collapse-toggle:hover { color: var(--text-primary); }
  .chevron { font-size: 10px; transition: transform 100ms; }
  .summarize-btn {
    padding: 3px 10px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    font-family: inherit;
  }
  .summarize-btn:hover { opacity: 0.9; }
  .card-body { padding: 10px 12px; }
  .summary-text {
    font-size: 13px;
    line-height: 1.5;
    color: var(--text-primary);
    margin: 0;
  }
  .loading, .placeholder {
    font-size: 12px;
    color: var(--text-disabled);
    margin: 0;
    font-style: italic;
  }
  .error {
    font-size: 12px;
    color: #d16969;
    margin: 0;
  }
</style>
