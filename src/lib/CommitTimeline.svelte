<script lang="ts">
  import { onMount } from 'svelte';
  import { recentCommits, selectedCommitOid, commitFiles, tick } from './stores';
  import { loadRecentCommits, selectCommit, openCommitFileDiff } from './actions';

  onMount(() => {
    loadRecentCommits();
  });

  function relativeTime(isoString: string): string {
    const now = Date.now();
    const then = new Date(isoString).getTime();
    const diff = Math.floor((now - then) / 1000);
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(isoString).toLocaleDateString();
  }

  function shortOid(oid: string): string {
    return oid.slice(0, 7);
  }

  function firstLine(msg: string): string {
    return msg.split('\n')[0].slice(0, 60);
  }

  function handleFileClick(repoPath: string, commitOid: string, filePath: string) {
    openCommitFileDiff(repoPath, commitOid, filePath);
  }

  // Reactive: re-evaluate relative timestamps
  $: void $tick;
</script>

<div class="timeline">
  {#if $recentCommits.length === 0}
    <div class="empty">No commits found</div>
  {:else}
    {#each $recentCommits as commit (commit.oid)}
      <button
        class="commit-entry"
        class:selected={$selectedCommitOid === commit.oid}
        on:click={() => selectCommit(commit.oid)}
      >
        <div class="commit-header">
          <span class="commit-msg">{firstLine(commit.message)}</span>
          {#if commit.session_id}
            <span class="session-badge" title="Claude Code session">🤖</span>
          {/if}
        </div>
        <div class="commit-meta">
          <span class="commit-oid">{shortOid(commit.oid)}</span>
          <span class="commit-stats">
            {#if commit.additions > 0}<span class="add">+{commit.additions}</span>{/if}
            {#if commit.deletions > 0}<span class="del">-{commit.deletions}</span>{/if}
          </span>
          <span class="commit-time">{relativeTime(commit.timestamp)}</span>
        </div>
        {#if commit.author}
          <div class="commit-author">{commit.author}</div>
        {/if}
      </button>

      {#if $selectedCommitOid === commit.oid}
        <div class="commit-files">
          {#each $commitFiles as file (file.id)}
            <button class="file-entry" on:click|stopPropagation={() => handleFileClick(commit.repo_path, commit.oid, file.file_path)}>
              <span class="file-status" class:added={file.status === 'added'} class:deleted={file.status === 'deleted'}>{file.status[0].toUpperCase()}</span>
              <span class="file-path">{file.file_path}</span>
              <span class="file-stats">
                {#if file.additions > 0}<span class="add">+{file.additions}</span>{/if}
                {#if file.deletions > 0}<span class="del">-{file.deletions}</span>{/if}
              </span>
            </button>
          {/each}
        </div>
      {/if}
    {/each}
  {/if}
</div>

<style>
  .timeline {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .empty {
    padding: 24px 16px;
    text-align: center;
    color: var(--text-disabled);
    font-size: 12px;
  }
  .commit-entry {
    display: block;
    width: 100%;
    text-align: left;
    padding: 8px 12px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    cursor: pointer;
    color: var(--text-primary);
    font-family: inherit;
  }
  .commit-entry:hover { background: var(--bg-hover); }
  .commit-entry.selected { background: var(--bg-active); }
  .commit-header {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 2px;
  }
  .commit-msg {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }
  .session-badge {
    font-size: 11px;
    flex-shrink: 0;
  }
  .commit-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-disabled);
  }
  .commit-oid {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 10px;
    color: var(--text-disabled);
  }
  .commit-stats { display: flex; gap: 4px; }
  .add { color: #4ec9b0; }
  .del { color: #d16969; }
  .commit-time { margin-left: auto; }
  .commit-author {
    font-size: 10px;
    color: var(--text-disabled);
    margin-top: 1px;
  }
  .commit-files {
    padding: 2px 0 4px 12px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border-subtle);
  }
  .file-entry {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 3px 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: inherit;
    font-size: 11px;
    color: var(--text-secondary);
    text-align: left;
  }
  .file-entry:hover { background: var(--bg-hover); color: var(--text-primary); }
  .file-status {
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    font-weight: 600;
    border-radius: 2px;
    background: var(--bg-active);
    flex-shrink: 0;
  }
  .file-status.added { color: #4ec9b0; }
  .file-status.deleted { color: #d16969; }
  .file-path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 10px;
  }
  .file-stats {
    display: flex;
    gap: 3px;
    font-size: 10px;
    flex-shrink: 0;
  }
</style>
