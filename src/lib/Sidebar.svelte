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
    background: var(--bg-base);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100%;
    transition: width var(--duration-slow) var(--ease-out);
  }
</style>
