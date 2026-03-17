<script lang="ts">
  import { annotations, activeAnnotationLine } from './stores';
  import { saveAnnotation, deleteAnnotation } from './actions';

  export let lineNumber: number;
  export let filePath: string;
  export let commitHash: string | null = null;

  let text = '';

  // Load existing annotation text if editing
  $: existing = ($annotations ?? []).find(a => a.line_number === lineNumber);
  $: if (existing) {
    text = existing.annotation_text;
  }

  async function handleSave() {
    if (!text.trim()) return;
    await saveAnnotation(filePath, lineNumber, text.trim(), commitHash ?? undefined);
    activeAnnotationLine.set(null);
  }

  async function handleDelete() {
    if (existing) {
      await deleteAnnotation(existing.id, filePath);
    }
    activeAnnotationLine.set(null);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      activeAnnotationLine.set(null);
    }
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSave();
    }
  }
</script>

<div class="annotation-popover" on:click|stopPropagation on:keydown={() => {}} role="region" tabindex="-1">
  <textarea
    bind:value={text}
    placeholder="Add annotation... (Cmd+Enter to save)"
    class="annotation-input"
    on:keydown={handleKeydown}
    rows="3"
  ></textarea>
  <div class="annotation-actions">
    <button class="save-btn" on:click={handleSave} disabled={!text.trim()}>Save</button>
    {#if existing}
      <button class="delete-btn" on:click={handleDelete}>Delete</button>
    {/if}
    <button class="cancel-btn" on:click={() => activeAnnotationLine.set(null)}>Cancel</button>
    <span class="hint">⌘↩ save · Esc cancel</span>
  </div>
</div>

<style>
  .annotation-popover {
    padding: 8px 12px 8px 90px;
    background: var(--bg-elevated);
    border-top: 1px solid var(--accent);
    border-bottom: 1px solid var(--accent);
  }
  .annotation-input {
    width: 100%;
    padding: 8px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-family: inherit;
    font-size: 12px;
    resize: vertical;
    outline: none;
  }
  .annotation-input:focus {
    border-color: var(--accent);
  }
  .annotation-input::placeholder {
    color: var(--text-disabled);
  }
  .annotation-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
  }
  .save-btn, .delete-btn, .cancel-btn {
    padding: 3px 10px;
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    font-family: inherit;
    background: var(--bg-active);
    color: var(--text-secondary);
  }
  .save-btn {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .save-btn:disabled { opacity: 0.5; cursor: default; }
  .save-btn:hover:not(:disabled) { opacity: 0.9; }
  .delete-btn { color: #d16969; border-color: #d16969; }
  .delete-btn:hover { background: rgba(209, 105, 105, 0.1); }
  .cancel-btn:hover { background: var(--bg-hover); }
  .hint {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-disabled);
  }
</style>
