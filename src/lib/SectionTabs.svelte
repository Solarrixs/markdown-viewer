<script lang="ts">
  import { currentSection, sidebarViewMode } from './stores';
  import { switchSection, toggleSidebarView } from './actions';
  import { sections } from './registry';
</script>

<div class="section-tabs">
  {#each sections as section}
    <button
      class="tab"
      class:active={$currentSection === section.id}
      on:click={() => switchSection(section.id)}
    >
      {section.label}
    </button>
  {/each}
  <button
    class="tab view-toggle"
    class:active={$sidebarViewMode === 'timeline'}
    on:click={toggleSidebarView}
    title="Toggle timeline view (G V)"
  >
    ⏱
  </button>
</div>

<style>
  .section-tabs {
    display: flex;
    gap: 2px;
    padding: 8px 8px 0;
    border-bottom: 1px solid var(--border);
  }
  .tab {
    flex: 1;
    padding: 6px 8px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px 4px 0 0;
    transition: all var(--duration-normal) ease;
  }
  .tab:hover {
    color: var(--text-primary);
    background: var(--bg-elevated);
  }
  .tab.active {
    color: var(--text-heading);
    background: var(--bg-elevated);
    border-bottom: 2px solid var(--accent);
  }
  .view-toggle {
    flex: 0;
    padding: 6px 10px;
    font-size: 13px;
  }
</style>
