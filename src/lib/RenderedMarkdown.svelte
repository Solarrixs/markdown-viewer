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

  $: rendered = sanitizeHtml(md.render($fileContent));

  export let containerEl: HTMLDivElement = undefined!;

  // Inject heading IDs only when rendered content changes
  let lastRendered = '';
  $: if (rendered !== lastRendered) {
    lastRendered = rendered;
    // Wait for DOM update before injecting IDs
    requestAnimationFrame(() => {
      if (!containerEl) return;
      const headings = containerEl.querySelectorAll('h1, h2, h3, h4');
      headings.forEach((el, i) => {
        el.id = `heading-${i}`;
      });
    });
  }

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

  onMount(() => {
    containerEl.addEventListener('click', handleCheckboxClick);
    // Restore scroll position from before the toggle
    const scroller = getScrollParent();
    if (scroller) {
      requestAnimationFrame(() => {
        scroller.scrollTop = get(scrollRatio) * (scroller.scrollHeight - scroller.clientHeight);
      });
    }
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
  {@html rendered}
</div>

<style>
  .markdown-body {
    max-width: 720px;
    margin: 0 auto;
    padding: 32px 24px;
    color: #eee;
    font-size: 15px;
    line-height: 1.7;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .markdown-body :global(h1) { font-size: 28px; font-weight: 600; margin: 32px 0 16px; color: #fff; border-bottom: 1px solid #3a3a3a; padding-bottom: 8px; }
  .markdown-body :global(h2) { font-size: 22px; font-weight: 600; margin: 28px 0 12px; color: #fff; }
  .markdown-body :global(h3) { font-size: 18px; font-weight: 600; margin: 24px 0 8px; color: #f0f0f0; }
  .markdown-body :global(h4) { font-size: 15px; font-weight: 600; margin: 20px 0 8px; color: #e0e0e0; }
  .markdown-body :global(p) { margin: 0 0 16px; }
  .markdown-body :global(a) { color: #5b9bd5; text-decoration: none; }
  .markdown-body :global(a:hover) { text-decoration: underline; }
  .markdown-body :global(ul), .markdown-body :global(ol) { padding-left: 24px; margin: 0 0 16px; }
  .markdown-body :global(li) { margin: 4px 0; }
  .markdown-body :global(blockquote) { border-left: 3px solid #555; padding-left: 16px; color: #bbb; margin: 0 0 16px; }
  .markdown-body :global(code) { background: #1e1e1e; padding: 2px 6px; border-radius: 3px; font-size: 13px; font-family: 'JetBrains Mono', 'Fira Code', monospace; }
  .markdown-body :global(pre.hljs) { background: #1e1e1e; border-radius: 6px; padding: 16px; overflow-x: auto; margin: 0 0 16px; }
  .markdown-body :global(pre.hljs code) { background: transparent; padding: 0; }
  .markdown-body :global(table) { width: 100%; border-collapse: collapse; margin: 0 0 16px; }
  .markdown-body :global(th), .markdown-body :global(td) { border: 1px solid #333; padding: 8px 12px; text-align: left; }
  .markdown-body :global(th) { background: #1e1e1e; font-weight: 600; }
  .markdown-body :global(hr) { border: none; border-top: 1px solid #333; margin: 24px 0; }
  .markdown-body :global(img) { max-width: 100%; border-radius: 4px; }
  .markdown-body :global(strong) { color: #fff; }
  /* Task list checkbox styles */
  .markdown-body :global(.task-list-item) { list-style: none; margin-left: -24px; padding-left: 0; }
  .markdown-body :global(.task-list-item input[type="checkbox"]) {
    margin-right: 8px;
    width: 16px;
    height: 16px;
    accent-color: #5b9bd5;
    cursor: pointer;
    vertical-align: middle;
  }
</style>
