<script lang="ts">
  import { activeFilePath, editText, fileContent, savedIndicator } from './stores';
  import { saveContent } from './actions';

  let loadedPath = '';
  let textarea: HTMLTextAreaElement;

  // Only sync from fileContent when the file changes (not during editing)
  $: if ($activeFilePath !== loadedPath) {
    $editText = $fileContent;
    loadedPath = $activeFilePath || '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      $editText = $editText.substring(0, start) + '  ' + $editText.substring(end);
      setTimeout(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 2;
      }, 0);
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault();
      save();
    }
  }

  async function save() {
    if (!$activeFilePath) return;
    try {
      await saveContent($activeFilePath, $editText);
    } catch (e) {
      console.error('Save failed:', e);
    }
  }

  function handleBlur() {
    if ($editText !== $fileContent) {
      save();
    }
  }
</script>

<div class="editor-container">
  <div class="editor-header">
    <span class="mode-label">EDIT MODE</span>
    <span class="hint">
      {#if $savedIndicator}
        <span class="saved-flash">Saved</span>
      {:else}
        Cmd+S to save &middot; Cmd+E to exit
      {/if}
    </span>
  </div>
  <textarea
    bind:this={textarea}
    bind:value={$editText}
    on:keydown={handleKeydown}
    on:blur={handleBlur}
    spellcheck="false"
    class="editor-textarea"
  ></textarea>
</div>

<style>
  .editor-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-width: 720px;
    margin: 0 auto;
    padding: 0 24px;
  }
  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid #333;
    margin-bottom: 8px;
  }
  .mode-label {
    font-size: 11px;
    font-weight: 600;
    color: #5b9bd5;
    letter-spacing: 1px;
  }
  .hint {
    font-size: 11px;
    color: #999;
  }
  .editor-textarea {
    flex: 1;
    width: 100%;
    background: transparent;
    border: none;
    color: #eee;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 14px;
    line-height: 1.6;
    resize: none;
    outline: none;
    tab-size: 2;
    padding: 16px 0;
  }
  .saved-flash {
    color: #4ec9b0;
    font-weight: 500;
  }
</style>
