<script lang="ts">
  import { onDestroy, afterUpdate } from 'svelte';
  import { fileContent } from './stores';

  export let containerEl: HTMLDivElement;
  export let scrollParent: HTMLElement;

  interface TocEntry {
    id: string;
    text: string;
    level: number;
  }

  let entries: TocEntry[] = [];
  let activeId: string = '';
  let observer: IntersectionObserver | null = null;
  let lastHeadingCount = -1;
  let lastContentRef = '';

  function extractHeadings() {
    if (!containerEl) return;

    // Dirty check: skip if content hasn't changed
    const content = $fileContent;
    if (content === lastContentRef) return;
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

    entries = Array.from(headings).map((el) => ({
      id: el.id,
      text: el.textContent || '',
      level: parseInt(el.tagName[1]),
    }));
    setupObserver();
  }

  function setupObserver() {
    if (observer) observer.disconnect();
    if (!scrollParent || entries.length === 0) return;

    // Track which headings are visible, pick the topmost
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
        // Pick the topmost visible heading by DOM order
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
      const el = containerEl.querySelector(`#${e.id}`);
      if (el) observer.observe(el);
    }
  }

  function jumpTo(id: string) {
    const el = containerEl.querySelector(`#${id}`);
    if (el) el.scrollIntoView({ behavior: 'instant' as ScrollBehavior });
  }

  afterUpdate(extractHeadings);

  onDestroy(() => {
    if (observer) observer.disconnect();
  });
</script>

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

<style>
  .toc {
    position: fixed;
    right: 16px;
    top: 50%;
    transform: translateY(-50%);
    max-width: 200px;
    max-height: 60vh;
    overflow-y: auto;
    opacity: 0.15;
    transition: opacity 200ms ease, background-color 200ms ease;
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
    background: rgba(26, 26, 26, 0.9);
  }
  .toc-entry {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: #888;
    font-size: 11px;
    line-height: 1.4;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .toc-entry:hover {
    color: #ccc;
    background: rgba(255, 255, 255, 0.05);
  }
  .toc-entry.active {
    color: #5b9bd5;
  }
</style>
