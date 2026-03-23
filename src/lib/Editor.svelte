<script lang="ts">
  import { onMount, onDestroy, afterUpdate } from 'svelte';
  import { get } from 'svelte/store';
  import { activeFilePath, editText, fileContent, savedIndicator, scrollRatio } from './stores';
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

  let focusPath: string | null = null;
  let focusContent: string = '';

  function handleFocus() {
    focusPath = $activeFilePath;
    focusContent = $fileContent;
  }

  function handleBlur() {
    if ($activeFilePath !== focusPath) return;
    // External reload changed fileContent but user hasn't typed anything —
    // saving would overwrite the fresh disk content with stale editor text.
    if ($fileContent !== focusContent && $editText === focusContent) return;
    if ($editText !== $fileContent) {
      save();
    }
  }

  onMount(() => {
    if (textarea) {
      const ratio = get(scrollRatio);
      // Double rAF ensures browser has completed layout with full textarea content
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          textarea.scrollTop = ratio * (textarea.scrollHeight - textarea.clientHeight);
          // Place cursor at the approximate line
          const lines = $editText.split('\n');
          const totalLines = lines.length;
          const targetLine = Math.round(ratio * totalLines);
          let charPos = 0;
          for (let i = 0; i < targetLine && i < lines.length; i++) {
            charPos += lines[i].length + 1;
          }
          textarea.selectionStart = textarea.selectionEnd = charPos;
          textarea.focus();
        });
      });
    }
  });

  onDestroy(() => {
    if (textarea && textarea.scrollHeight > textarea.clientHeight) {
      scrollRatio.set(textarea.scrollTop / (textarea.scrollHeight - textarea.clientHeight));
    }
  });
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
    on:focus={handleFocus}
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
    padding: 0 24px;
  }
  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    margin-bottom: 8px;
  }
  .mode-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    letter-spacing: 1px;
  }
  .hint {
    font-size: 11px;
    color: var(--text-secondary);
  }
  .editor-textarea {
    flex: 1;
    width: 100%;
    background: transparent;
    border: none;
    color: var(--text-primary);
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
