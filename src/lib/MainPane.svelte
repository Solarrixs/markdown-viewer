<script lang="ts">
  import TabBar from './TabBar.svelte';
  import RenderedMarkdown from './RenderedMarkdown.svelte';
  import DiffView from './DiffView.svelte';
  import Editor from './Editor.svelte';
  import { activeFilePath, editMode, showDiff } from './stores';
</script>

<div class="main-pane">
  <TabBar />
  <div class="content">
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
      <div class="reader-scroll">
        <RenderedMarkdown />
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
    background: #1e1e1e;
  }
  .content {
    flex: 1;
    overflow-y: auto;
  }
  .reader-scroll {
    height: 100%;
    overflow-y: auto;
  }
  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #555;
  }
  .welcome h2 { color: #444; font-size: 24px; font-weight: 300; margin-bottom: 8px; }
  .welcome p { font-size: 13px; margin-bottom: 24px; }
  .shortcuts { text-align: center; }
  .shortcuts p { font-size: 12px; color: #444; margin: 4px 0; }
  .shortcuts :global(kbd) {
    background: #2a2a2a;
    border: 1px solid #333;
    border-radius: 3px;
    padding: 1px 5px;
    font-family: monospace;
    font-size: 11px;
    color: #999;
  }
</style>
