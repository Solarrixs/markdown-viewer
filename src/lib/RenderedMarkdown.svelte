<script lang="ts">
  import { onMount, onDestroy, afterUpdate } from 'svelte';
  import { get } from 'svelte/store';
  import { fileContent, activeFilePath, scrollRatio } from './stores';
  import { saveContent } from './actions';
  import MarkdownIt from 'markdown-it';
  // @ts-ignore — no types available
  import taskLists from 'markdown-it-task-lists';
  import hljs from 'highlight.js';

  const md = MarkdownIt({
    html: true,
    linkify: true,
    typographer: true,
    highlight: function (str: string, lang: string) {
      if (lang && hljs.getLanguage(lang)) {
        try {
          return '<pre class="hljs"><code>' + hljs.highlight(str, { language: lang }).value + '</code></pre>';
        } catch (_) {}
      }
      return '<pre class="hljs"><code>' + md.utils.escapeHtml(str) + '</code></pre>';
    }
  }).use(taskLists, { enabled: true });

  // Strip HTML comments and dangerous tags (script, iframe, etc.) from rendered output
  function sanitizeHtml(html: string): string {
    return html
      .replace(/<!--[\s\S]*?-->/g, '')
      .replace(/<\s*\/?\s*(script|iframe|object|embed|form|link|meta|style)[^>]*>/gi, '');
  }

  // Parse YAML frontmatter (--- delimited block at start of file)
  function parseFrontmatter(content: string): { meta: Record<string, string> | null; body: string } {
    const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?/);
    if (!match) return { meta: null, body: content };
    const raw = match[1];
    const meta: Record<string, string> = {};
    for (const line of raw.split('\n')) {
      const idx = line.indexOf(':');
      if (idx > 0) {
        const key = line.slice(0, idx).trim();
        const val = line.slice(idx + 1).trim();
        if (key) meta[key] = val;
      }
    }
    return { meta: Object.keys(meta).length > 0 ? meta : null, body: content.slice(match[0].length) };
  }

  $: ({ meta: frontmatter, body: markdownBody } = parseFrontmatter($fileContent));
  $: rendered = sanitizeHtml(md.render(markdownBody));

  export let containerEl: HTMLDivElement = undefined!;

  // Inject heading IDs synchronously in afterUpdate
  afterUpdate(() => {
    if (!containerEl) return;
    const headings = containerEl.querySelectorAll('h1, h2, h3, h4');
    headings.forEach((el, i) => {
      if (!el.id) el.id = `heading-${i}`;
    });
  });

  // Find the index of a checkbox in the markdown source, skipping matches inside code blocks
  function findCheckboxIndices(content: string): number[] {
    const indices: number[] = [];
    // Match fenced code blocks and inline code to skip them
    const codeBlockRegex = /(`{3,})[^`]*?\1|`[^`\n]+`/g;
    const codeRanges: [number, number][] = [];
    let codeMatch: RegExpExecArray | null;
    while ((codeMatch = codeBlockRegex.exec(content)) !== null) {
      codeRanges.push([codeMatch.index, codeMatch.index + codeMatch[0].length]);
    }

    const checkboxRegex = /- \[([ xX])\]/g;
    let match: RegExpExecArray | null;
    while ((match = checkboxRegex.exec(content)) !== null) {
      const pos = match.index;
      const inCode = codeRanges.some(([start, end]) => pos >= start && pos < end);
      if (!inCode) {
        indices.push(pos);
      }
    }
    return indices;
  }

  function handleCheckboxClick(e: Event) {
    const target = e.target as HTMLElement;
    if (target.tagName !== 'INPUT' || target.getAttribute('type') !== 'checkbox') return;

    const input = target as HTMLInputElement;
    const allCheckboxes = containerEl.querySelectorAll('input[type="checkbox"].task-list-item-checkbox');
    const checkboxIndex = Array.from(allCheckboxes).indexOf(input);
    if (checkboxIndex === -1) return;

    const content = get(fileContent);
    const positions = findCheckboxIndices(content);
    if (checkboxIndex >= positions.length) return;

    const pos = positions[checkboxIndex];
    const current = content[pos + 3]; // the character inside [ ]
    const replacement = current === ' ' ? '- [x]' : '- [ ]';
    const newContent = content.slice(0, pos) + replacement + content.slice(pos + 5);

    if (newContent !== content) {
      fileContent.set(newContent);
      const path = get(activeFilePath);
      if (path) {
        saveContent(path, newContent);
      }
    }
  }

  function getScrollParent(): HTMLElement | null {
    return containerEl?.closest('.reader-scroll') as HTMLElement | null;
  }

  function restoreScroll() {
    const scroller = getScrollParent();
    if (!scroller) return;
    const ratio = get(scrollRatio);
    // Wait for browser layout so scrollHeight is accurate
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        scroller.scrollTop = ratio * (scroller.scrollHeight - scroller.clientHeight);
      });
    });
  }

  onMount(() => {
    containerEl.addEventListener('click', handleCheckboxClick);
    restoreScroll();
  });

  onDestroy(() => {
    // Save scroll position before being destroyed by the toggle
    const scroller = getScrollParent();
    if (scroller && scroller.scrollHeight > scroller.clientHeight) {
      scrollRatio.set(scroller.scrollTop / (scroller.scrollHeight - scroller.clientHeight));
    }
    if (containerEl) {
      containerEl.removeEventListener('click', handleCheckboxClick);
    }
  });
</script>

<div class="markdown-body" bind:this={containerEl}>
  {#if frontmatter}
    <div class="frontmatter">
      {#each Object.entries(frontmatter) as [key, value]}
        <div class="frontmatter-row">
          <span class="frontmatter-key">{key}</span>
          <span class="frontmatter-value">{value}</span>
        </div>
      {/each}
    </div>
  {/if}
  {@html rendered}
</div>

<style>
  .markdown-body {
    padding: 32px 0;
    color: var(--text-primary);
    font-size: 15px;
    line-height: 1.7;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .frontmatter {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px 16px;
    margin-bottom: 24px;
    font-size: 13px;
  }
  .frontmatter-row {
    display: flex;
    gap: 12px;
    padding: 3px 0;
  }
  .frontmatter-key {
    color: var(--text-secondary);
    min-width: 100px;
    flex-shrink: 0;
  }
  .frontmatter-key::after {
    content: ':';
  }
  .frontmatter-value {
    color: var(--text-primary);
  }
  .markdown-body :global(h1) { font-size: 28px; font-weight: 600; margin: 32px 0 16px; color: var(--text-heading); border-bottom: 1px solid var(--border); padding-bottom: 8px; }
  .markdown-body :global(h2) { font-size: 22px; font-weight: 600; margin: 28px 0 12px; color: var(--text-heading); }
  .markdown-body :global(h3) { font-size: 18px; font-weight: 600; margin: 24px 0 8px; color: var(--text-heading); }
  .markdown-body :global(h4) { font-size: 15px; font-weight: 600; margin: 20px 0 8px; color: var(--text-primary); }
  .markdown-body :global(p) { margin: 0 0 16px; }
  .markdown-body :global(a) { color: var(--accent); text-decoration: none; }
  .markdown-body :global(a:hover) { text-decoration: underline; }
  .markdown-body :global(ul), .markdown-body :global(ol) { padding-left: 24px; margin: 0 0 16px; }
  .markdown-body :global(li) { margin: 4px 0; }
  .markdown-body :global(blockquote) { border-left: 3px solid var(--text-secondary); padding-left: 16px; color: var(--text-secondary); margin: 0 0 16px; }
  .markdown-body :global(code) { background: var(--bg-elevated); padding: 2px 6px; border-radius: 3px; font-size: 13px; font-family: 'JetBrains Mono', 'Fira Code', monospace; }
  .markdown-body :global(pre.hljs) { background: var(--bg-elevated); border-radius: 6px; padding: 16px; overflow-x: auto; margin: 0 0 16px; }
  .markdown-body :global(pre.hljs code) { background: transparent; padding: 0; }
  .markdown-body :global(table) { width: 100%; border-collapse: collapse; margin: 0 0 16px; }
  .markdown-body :global(th), .markdown-body :global(td) { border: 1px solid var(--border); padding: 8px 12px; text-align: left; }
  .markdown-body :global(th) { background: var(--bg-elevated); font-weight: 600; }
  .markdown-body :global(hr) { border: none; border-top: 1px solid var(--border); margin: 24px 0; }
  .markdown-body :global(img) { max-width: 100%; border-radius: 4px; }
  .markdown-body :global(strong) { color: var(--text-heading); }
  /* Task list checkbox styles */
  .markdown-body :global(.task-list-item) { list-style: none; margin-left: -24px; padding-left: 0; }
  .markdown-body :global(.task-list-item input[type="checkbox"]) {
    margin-right: 8px;
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
    cursor: pointer;
    vertical-align: middle;
  }
</style>
