<script lang="ts">
  import { currentSection } from './stores';
  import type { Section } from './stores';
  import { switchSection } from './actions';

  const sections: { id: Section; label: string }[] = [
    { id: 'inbox', label: 'Inbox' },
    { id: 'pinned', label: 'Pinned' },
    { id: 'reminders', label: 'Reminders' },
    { id: 'archive', label: 'Archive' },
  ];
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
</div>

<style>
  .section-tabs {
    display: flex;
    gap: 2px;
    padding: 8px 8px 0;
    border-bottom: 1px solid #3a3a3a;
  }
  .tab {
    flex: 1;
    padding: 6px 8px;
    background: transparent;
    border: none;
    color: #bbb;
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px 4px 0 0;
    transition: all 0.15s;
  }
  .tab:hover {
    color: #e0e0e0;
    background: #1e1e1e;
  }
  .tab.active {
    color: #fff;
    background: #1e1e1e;
    border-bottom: 2px solid #5b9bd5;
  }
</style>
