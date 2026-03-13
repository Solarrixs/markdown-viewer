<script lang="ts">
  import { fileContent } from './stores';
  import MarkdownIt from 'markdown-it';
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
  });

  $: rendered = md.render($fileContent);
</script>

<div class="markdown-body">
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
</style>
