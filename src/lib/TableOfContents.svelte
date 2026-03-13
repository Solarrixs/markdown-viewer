<script lang="ts">
  import { onDestroy, afterUpdate } from 'svelte';
  import { fileContent } from './stores';

  export let containerEl: HTMLDivElement;
  export let scrollParent: HTMLElement;

  interface TocEntry {
    id: string;
    text: string;
    level: number;
    el: Element;
  }

  let entries: TocEntry[] = [];
  let activeId: string = '';
  let observer: IntersectionObserver | null = null;
  let lastHeadingCount = -1;
  let lastContentRef = '';

  function extractHeadings() {
    if (!containerEl) return;

    // Dirty check: skip if content hasn't changed (unless entries are empty)
    const content = $fileContent;
    if (content === lastContentRef && entries.length > 0) return;
    lastContentRef = content;

    const headings = containerEl.querySelectorAll('h1, h2, h3, h4');
    if (headings.length === lastHeadingCount) {
      // Check if text actually changed
      let same = true;
      headings.forEach((el, i) => {
        if (entries[i]?.text !== (el.textContent || '')) same = false;
      });
      if (same) return;
    }
    lastHeadingCount = headings.length;

    // Assign IDs to headings that don't have them yet (in case our
    // afterUpdate runs before RenderedMarkdown's heading ID assignment)
    headings.forEach((el, i) => {
      if (!el.id) el.id = `heading-${i}`;
    });

    entries = Array.from(headings)
      .map((el) => ({
        id: el.id,
        text: el.textContent || '',
        level: parseInt(el.tagName[1]),
        el,
      }));
    if (entries.length > 0) setupObserver();
  }

  function setupObserver() {
    if (observer) observer.disconnect();
    if (!scrollParent || entries.length === 0) return;

    const visibleIds = new Set<string>();

    observer = new IntersectionObserver(
      (intersections) => {
        for (const entry of intersections) {
          if (entry.isIntersecting) {
            visibleIds.add(entry.target.id);
          } else {
            visibleIds.delete(entry.target.id);
          }
        }
        for (const e of entries) {
          if (visibleIds.has(e.id)) {
            activeId = e.id;
            break;
          }
        }
      },
      {
        root: scrollParent,
        rootMargin: '0px 0px -70% 0px',
        threshold: 0,
      }
    );

    for (const e of entries) {
      observer.observe(e.el);
    }
  }

  function jumpTo(id: string) {
    const entry = entries.find(e => e.id === id);
    if (entry) entry.el.scrollIntoView({ behavior: 'instant' as ScrollBehavior });
  }

  afterUpdate(extractHeadings);

  onDestroy(() => {
    if (observer) observer.disconnect();
  });
</script>

{#if entries.length > 0}
<nav class="toc">
  {#each entries as entry}
    <button
      class="toc-entry"
      class:active={entry.id === activeId}
      style="padding-left: {(entry.level - 1) * 12}px"
      on:click={() => jumpTo(entry.id)}
    >
      {entry.text}
    </button>
  {/each}
</nav>
{/if}

<style>
  .toc {
    position: fixed;
    right: 16px;
    top: 50%;
    transform: translateY(-50%);
    width: 180px;
    max-height: 60vh;
    overflow-y: auto;
    opacity: 0.2;
    transition: opacity 0.2s ease;
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px;
    border-radius: 6px;
    z-index: 10;
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
  .toc::-webkit-scrollbar {
    display: none;
  }
  .toc:hover {
    opacity: 1;
    background: var(--bg-elevated);
  }
  .toc-entry {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 11px;
    line-height: 1.4;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .toc-entry:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }
  .toc-entry.active {
    color: var(--accent);
  }
</style>
