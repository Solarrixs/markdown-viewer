<script lang="ts">
  import TabBar from './TabBar.svelte';
  import NoteToolbar from './NoteToolbar.svelte';
  import RenderedMarkdown from './RenderedMarkdown.svelte';
  import DiffView from './DiffView.svelte';
  import Editor from './Editor.svelte';
  import TableOfContents from './TableOfContents.svelte';
  import FindBar from './FindBar.svelte';
  import MarkdownIt from 'markdown-it';
  // @ts-ignore
  import taskLists from 'markdown-it-task-lists';
  import { activeFilePath, editMode, showDiff, showToc, findBarOpen, splitPath, splitContent, activeSplit } from './stores';
  import { closeSplit } from './actions';

  const splitMd = MarkdownIt({ html: true, linkify: true, typographer: true }).use(taskLists);
  $: splitRendered = $splitContent ? splitMd.render($splitContent) : '';

  let markdownEl: HTMLDivElement;
  let readerScrollEl: HTMLDivElement;
</script>

<div class="main-pane">
  <TabBar />
  <NoteToolbar />
  {#if $findBarOpen && $activeFilePath && !$editMode && markdownEl}
    <FindBar containerEl={markdownEl} />
  {/if}
  <div class="content" class:split={$splitPath}>
    <div class="pane left" class:active-pane={!$splitPath || $activeSplit === 'left'} class:toc-visible={$showToc && $activeFilePath && !$editMode && !$showDiff} on:click={() => activeSplit.set('left')} on:keydown={() => {}} role="region" tabindex="-1">
      {#if !$activeFilePath}
        <div class="welcome">
          <h2>MarkInbox</h2>
          <p>Select a file from the sidebar to view it</p>
          <div class="shortcuts">
            <p><kbd>J</kbd>/<kbd>K</kbd> navigate &middot; <kbd>Enter</kbd> open &middot; <kbd>E</kbd> archive</p>
            <p><kbd>Cmd+K</kbd> command palette &middot; <kbd>Cmd+E</kbd> edit</p>
          </div>
        </div>
      {:else if $editMode}
        <Editor />
      {:else if $showDiff}
        <DiffView />
      {:else}
        <div class="reader-scroll" bind:this={readerScrollEl}>
          <RenderedMarkdown bind:containerEl={markdownEl} />
          {#if $showToc && markdownEl}
            <TableOfContents containerEl={markdownEl} scrollParent={readerScrollEl} />
          {/if}
        </div>
      {/if}
    </div>
    {#if $splitPath}
      <div class="split-divider"></div>
      <div class="pane right" class:active-pane={$activeSplit === 'right'} on:click={() => activeSplit.set('right')} on:keydown={() => {}} role="region" tabindex="-1">
        <div class="split-header">
          <span class="split-filename">{$splitPath.split('/').pop()}</span>
          <button class="split-close" on:click|stopPropagation={closeSplit}>&times;</button>
        </div>
        <div class="reader-scroll">
          <div class="markdown-body split-body">
            {@html splitRendered}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .main-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    background: var(--bg-surface);
  }
  .content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .content.split {
    flex-direction: row;
  }
  .pane {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
    position: relative;
  }
  .pane.left, .pane.right {
    border-top: 2px solid transparent;
  }
  .pane.active-pane {
    border-top-color: var(--accent);
  }
  .pane.toc-visible {
    padding-right: 200px;
  }
  .split-divider {
    width: 1px;
    background: var(--border);
    flex-shrink: 0;
  }
  .split-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }
  .split-filename {
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .split-close {
    background: transparent;
    border: none;
    color: var(--text-disabled);
    font-size: 16px;
    cursor: pointer;
    padding: 0 4px;
  }
  .split-close:hover { color: var(--text-primary); }
  .split-body {
    padding: 32px 24px;
    color: var(--text-primary);
    font-size: 15px;
    line-height: 1.7;
  }
  .reader-scroll {
    height: 100%;
    overflow-y: auto;
    position: relative;
    padding: 0 24px;
  }
  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-disabled);
    user-select: none;
    -webkit-user-select: none;
  }
  .welcome h2 { color: var(--text-disabled); font-size: 24px; font-weight: 300; margin-bottom: 8px; }
  .welcome p { font-size: 13px; margin-bottom: 24px; }
  .shortcuts { text-align: center; }
  .shortcuts p { font-size: 12px; color: var(--text-disabled); margin: 4px 0; }
  .shortcuts :global(kbd) {
    background: var(--bg-active);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 5px;
    font-family: monospace;
    font-size: 11px;
    color: var(--text-secondary);
  }
</style>
