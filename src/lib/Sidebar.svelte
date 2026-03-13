<script lang="ts">
  import SectionTabs from './SectionTabs.svelte';
  import FileList from './FileList.svelte';
  import { sidebarVisible } from './stores';
</script>

{#if $sidebarVisible}
  <aside class="sidebar">
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
    border-right: 1px solid #3a3a3a;
    display: flex;
    flex-direction: column;
    height: 100%;
  }
</style>
