<script lang="ts">
  import SectionTabs from './SectionTabs.svelte';
  import FileList from './FileList.svelte';
  import { sidebarVisible } from './stores';
</script>

{#if $sidebarVisible}
  <aside class="sidebar">
    <div class="header">
      <h1 class="logo">MarkInbox</h1>
    </div>
    <SectionTabs />
    <FileList />
  </aside>
{/if}

<style>
  .sidebar {
    width: 260px;
    min-width: 200px;
    max-width: 400px;
    background: #161616;
    border-right: 1px solid #2a2a2a;
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .header {
    padding: 12px 16px 8px;
  }
  .logo {
    font-size: 14px;
    font-weight: 600;
    color: #e0e0e0;
    margin: 0;
    letter-spacing: 0.5px;
  }
</style>
