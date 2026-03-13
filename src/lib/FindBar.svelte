<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { findBarOpen } from './stores';

  export let containerEl: HTMLDivElement;

  let inputEl: HTMLInputElement;
  let query = '';
  let matches: HTMLElement[] = [];
  let currentIndex = 0;
  let totalMatches = 0;
  let debounceTimer: ReturnType<typeof setTimeout>;

  onMount(() => {
    setTimeout(() => inputEl?.focus(), 50);
  });

  onDestroy(() => {
    clearHighlights();
  });

  function clearHighlights() {
    if (!containerEl) return;
    const marks = containerEl.querySelectorAll('mark.find-match');
    const parents = new Set<Node>();
    marks.forEach(mark => {
      const parent = mark.parentNode;
      if (parent) {
        while (mark.firstChild) {
          parent.insertBefore(mark.firstChild, mark);
        }
        parent.removeChild(mark);
        parents.add(parent);
      }
    });
    parents.forEach(p => p.normalize());
    matches = [];
    currentIndex = 0;
    totalMatches = 0;
  }

  function performSearch(query: string) {
    clearHighlights();
    matches = [];
    currentIndex = 0;
    totalMatches = 0;

    if (!query || query.length === 0) return;

    const lowerQuery = query.toLowerCase();
    const walker = document.createTreeWalker(containerEl, NodeFilter.SHOW_TEXT);
    const textNodes: { node: Text; start: number }[] = [];

    let node: Text | null;
    while ((node = walker.nextNode() as Text | null)) {
      const text = node.textContent || '';
      const lowerText = text.toLowerCase();
      let idx = lowerText.indexOf(lowerQuery);
      while (idx !== -1) {
        textNodes.push({ node, start: idx });
        idx = lowerText.indexOf(lowerQuery, idx + 1);
      }
    }

    // Wrap matches in reverse order to preserve offsets
    for (let i = textNodes.length - 1; i >= 0; i--) {
      const { node: textNode, start } = textNodes[i];
      const range = document.createRange();
      range.setStart(textNode, start);
      range.setEnd(textNode, start + query.length);

      const mark = document.createElement('mark');
      mark.className = 'find-match';
      mark.dataset.matchIndex = String(i);
      range.surroundContents(mark);
    }

    matches = Array.from(containerEl.querySelectorAll('mark.find-match')) as HTMLElement[];
    totalMatches = matches.length;
    prevActiveMatch = null;

    if (totalMatches > 0) {
      currentIndex = 0;
      highlightCurrent();
    }
  }

  let prevActiveMatch: HTMLElement | null = null;

  function highlightCurrent() {
    if (prevActiveMatch) {
      prevActiveMatch.classList.remove('find-match-active');
    }
    const current = matches[currentIndex];
    if (current) {
      current.classList.add('find-match-active');
      current.scrollIntoView({ block: 'center', behavior: 'smooth' });
      prevActiveMatch = current;
    }
  }

  function nextMatch() {
    if (totalMatches === 0) return;
    currentIndex = (currentIndex + 1) % totalMatches;
    highlightCurrent();
  }

  function prevMatch() {
    if (totalMatches === 0) return;
    currentIndex = (currentIndex - 1 + totalMatches) % totalMatches;
    highlightCurrent();
  }

  function close() {
    clearHighlights();
    findBarOpen.set(false);
    query = '';
  }

  function handleInput() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      performSearch(query);
    }, 100);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
    } else if (e.key === 'Enter' && e.shiftKey) {
      e.preventDefault();
      prevMatch();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      nextMatch();
    }
  }
</script>

<div class="find-bar">
  <input
    bind:this={inputEl}
    bind:value={query}
    on:input={handleInput}
    on:keydown={handleKeydown}
    placeholder="Find in document..."
    class="find-input"
  />
  <span class="match-count">
    {#if totalMatches > 0}
      {currentIndex + 1} of {totalMatches}
    {:else if query.length > 0}
      No matches
    {/if}
  </span>
  <button class="nav-btn" on:click={prevMatch} title="Previous (Shift+Enter)" disabled={totalMatches === 0}>&uarr;</button>
  <button class="nav-btn" on:click={nextMatch} title="Next (Enter)" disabled={totalMatches === 0}>&darr;</button>
  <button class="close-btn" on:click={close} title="Close (Esc)">&times;</button>
</div>

<style>
  .find-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 12px;
    background: var(--bg-hover);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .find-input {
    flex: 1;
    padding: 4px 8px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
  }
  .find-input:focus { border-color: var(--accent); }
  .find-input::placeholder { color: var(--text-disabled); }
  .match-count {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    min-width: 60px;
    text-align: center;
  }
  .nav-btn {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 6px;
    line-height: 1;
  }
  .nav-btn:hover:not(:disabled) { background: var(--bg-active); color: var(--text-primary); }
  .nav-btn:disabled { opacity: 0.3; cursor: default; }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-disabled);
    font-size: 16px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }
  .close-btn:hover { color: var(--text-primary); }
  :global(mark.find-match) {
    background: rgba(91, 155, 213, 0.3);
    border-radius: 2px;
  }
  :global(mark.find-match.find-match-active) {
    background: rgba(91, 155, 213, 0.6);
  }
</style>
