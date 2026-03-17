<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { claudeSessions, reviewStatuses } from './stores';
  import { loadClaudeSessions, loadReviewStatuses, showToast } from './actions';
  import { generateFeedbackMarkdown } from './feedbackWriter';
  import type { AnnotationRecord, SessionInfo } from './stores';

  let selectedSessionId: string = '';
  let sending = false;
  let previewMode = false;
  let previewText = '';

  onMount(() => {
    loadClaudeSessions();
    loadReviewStatuses();
  });

  function statusIcon(status: string): string {
    switch (status) {
      case 'active': return '🟢';
      case 'idle': return '🟡';
      default: return '⚪';
    }
  }

  async function generatePreview() {
    const unsent = await invoke<AnnotationRecord[]>('get_unsent_annotations');
    previewText = generateFeedbackMarkdown(unsent, $reviewStatuses);
    previewMode = true;
  }

  async function sendFeedback() {
    if (!selectedSessionId) {
      showToast('Select a Claude session first');
      return;
    }

    sending = true;
    try {
      // Collect unsent annotations
      const unsent = await invoke<AnnotationRecord[]>('get_unsent_annotations');
      if (unsent.length === 0 && $reviewStatuses.length === 0) {
        showToast('No feedback to send');
        sending = false;
        return;
      }

      // Generate markdown
      const markdown = generateFeedbackMarkdown(unsent, $reviewStatuses);

      // Send to Claude
      await invoke('send_feedback_to_session', {
        sessionId: selectedSessionId,
        feedbackText: markdown,
      });

      // Mark annotations as sent
      const ids = unsent.map(a => a.id);
      if (ids.length > 0) {
        await invoke('mark_annotations_sent', { ids });
      }

      showToast(`Sent feedback (${unsent.length} annotations) to Claude`);
      previewMode = false;
    } catch (e) {
      showToast(`Failed: ${e}`);
    } finally {
      sending = false;
    }
  }
</script>

<div class="feedback-panel">
  <div class="panel-header">
    <h3>Send Feedback to Claude</h3>
  </div>

  <div class="panel-body">
    <label class="field-label">Session</label>
    <select class="session-select" bind:value={selectedSessionId}>
      <option value="">Select a session...</option>
      {#each $claudeSessions as session}
        <option value={session.session_id}>
          {statusIcon(session.status)} {session.session_id.slice(0, 8)}... ({session.status})
        </option>
      {/each}
    </select>
    <button class="refresh-btn" on:click={loadClaudeSessions} title="Refresh sessions">↻</button>

    {#if previewMode}
      <pre class="preview">{previewText}</pre>
    {/if}

    <div class="actions">
      <button class="preview-btn" on:click={generatePreview}>Preview</button>
      <button class="send-btn" on:click={sendFeedback} disabled={!selectedSessionId || sending}>
        {sending ? 'Sending...' : 'Send to Claude'}
      </button>
    </div>
  </div>
</div>

<style>
  .feedback-panel {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-elevated);
    margin: 12px 0;
    overflow: hidden;
  }
  .panel-header {
    padding: 8px 12px;
    background: var(--bg-active);
    border-bottom: 1px solid var(--border-subtle);
  }
  .panel-header h3 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .panel-body {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field-label {
    font-size: 11px;
    color: var(--text-disabled);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .session-select {
    padding: 6px 8px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 12px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    outline: none;
  }
  .session-select:focus { border-color: var(--accent); }
  .refresh-btn {
    align-self: flex-end;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 14px;
    cursor: pointer;
    padding: 2px 8px;
  }
  .refresh-btn:hover { background: var(--bg-hover); }
  .preview {
    padding: 8px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 11px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    color: var(--text-secondary);
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .preview-btn, .send-btn {
    padding: 5px 14px;
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    font-family: inherit;
  }
  .preview-btn {
    background: var(--bg-active);
    color: var(--text-secondary);
  }
  .preview-btn:hover { background: var(--bg-hover); }
  .send-btn {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .send-btn:disabled { opacity: 0.5; cursor: default; }
  .send-btn:hover:not(:disabled) { opacity: 0.9; }
</style>
