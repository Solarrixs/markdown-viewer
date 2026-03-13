<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { bulkOpenModalOpen } from './stores';
  import { openMultipleFilePaths, showToast } from './actions';

  let textValue = '';
  let loading = false;
  let textareaEl: HTMLTextAreaElement;

  $: if ($bulkOpenModalOpen) {
    textValue = '';
    loading = false;
    setTimeout(() => textareaEl?.focus(), 50);
  }

  function close() {
    bulkOpenModalOpen.set(false);
  }

  function parsePaths(text: string): string[] {
    return text
      .split(/[\n,]/)
      .map(p => p.trim())
      .filter(p => p.length > 0);
  }

  async function submit() {
    const paths = parsePaths(textValue);
    if (paths.length === 0) {
      showToast('No valid paths entered');
      return;
    }
    loading = true;
    close();
    await openMultipleFilePaths(paths);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    } else if (e.key === 'Enter' && e.metaKey) {
      e.preventDefault();
      submit();
    }
  }
</script>

{#if $bulkOpenModalOpen}
  <div class="backdrop" transition:fade={{ duration: 150 }} on:click={close} on:keydown={(e) => { if (e.key === 'Escape') close(); }} role="button" tabindex="-1">
    <div class="modal" transition:scale={{ start: 0.98, duration: 150 }} on:click|stopPropagation role="dialog" tabindex="-1" on:keydown={() => {}}>
      <div class="header">Open Multiple Files</div>
      <textarea
        bind:this={textareaEl}
        bind:value={textValue}
        on:keydown={handleKeydown}
        placeholder="Paste file paths, one per line or comma-separated&#10;&#10;/Users/you/notes/file1.md&#10;/Users/you/notes/file2.md"
        class="path-input"
        rows="8"
      ></textarea>
      <div class="footer">
        <span class="hint">Cmd+Enter to open</span>
        <div class="buttons">
          <button class="btn cancel" on:click={close}>Cancel</button>
          <button class="btn open" on:click={submit} disabled={loading}>
            {loading ? 'Opening...' : 'Open'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    padding-top: 100px;
    z-index: 100;
  }
  .modal {
    width: 500px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    height: fit-content;
  }
  .header {
    padding: 12px 16px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
  }
  .path-input {
    width: 100%;
    padding: 12px 16px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    font-family: 'SF Mono', 'Menlo', 'Monaco', monospace;
    line-height: 1.5;
    outline: none;
    resize: vertical;
    min-height: 120px;
  }
  .path-input::placeholder { color: var(--text-disabled); }
  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
  }
  .hint {
    font-size: 11px;
    color: var(--text-disabled);
  }
  .buttons {
    display: flex;
    gap: 8px;
  }
  .btn {
    padding: 6px 14px;
    border-radius: 4px;
    border: none;
    font-size: 12px;
    cursor: pointer;
  }
  .cancel {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
  .cancel:hover { background: var(--bg-active); }
  .open {
    background: var(--accent);
    color: #fff;
  }
  .open:hover { opacity: 0.9; }
  .open:disabled { opacity: 0.5; cursor: default; }
</style>
