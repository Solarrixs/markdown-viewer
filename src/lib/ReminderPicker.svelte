<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { reminderPickerOpen, activeFilePath } from './stores';
  import { setReminderAndArchive } from './actions';
  import { parseNaturalTime, reminderIsoString, reminderOptions, formatReminderPreview } from './utils';

  let nlInput = '';
  let inputEl: HTMLInputElement;

  $: if ($reminderPickerOpen) {
    nlInput = '';
    setTimeout(() => inputEl?.focus(), 50);
  }

  $: parsed = parseNaturalTime(nlInput);
  $: preview = parsed ? formatReminderPreview(parsed) : '';

  async function setReminder(time: string) {
    if (!$activeFilePath) return;
    try {
      await setReminderAndArchive($activeFilePath, time);
    } catch (e) {
      console.error('Failed to set reminder:', e);
    }
    close();
  }

  function submitNlReminder() {
    if (parsed) {
      setReminder(reminderIsoString(parsed));
    }
  }

  function close() {
    reminderPickerOpen.set(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
    if (e.key === 'Enter' && parsed) {
      e.preventDefault();
      submitNlReminder();
    }
  }
</script>

{#if $reminderPickerOpen}
  <div class="backdrop" transition:fade={{ duration: 150 }} on:click={close} on:keydown={handleKeydown} role="button" tabindex="-1">
    <div class="picker" transition:scale={{ start: 0.98, duration: 150 }} on:click|stopPropagation on:keydown={() => {}} role="dialog" tabindex="-1">
      <div class="header">Set Reminder</div>
      <div class="nl-input-wrap">
        <input
          bind:this={inputEl}
          bind:value={nlInput}
          on:keydown={handleKeydown}
          placeholder="e.g. 2h, 1d, tomorrow, next week..."
          class="nl-input"
        />
        {#if preview}
          <div class="preview">{preview}</div>
        {/if}
      </div>
      <div class="presets-label">Quick options</div>
      {#each reminderOptions as opt}
        <button class="option" on:click={() => setReminder(opt.getTime())}>
          {opt.label}
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    padding-top: 160px;
    z-index: 100;
  }
  .picker {
    width: 300px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    height: fit-content;
  }
  .header {
    padding: 12px 16px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border);
  }
  .nl-input-wrap {
    padding: 10px 12px 6px;
  }
  .nl-input {
    width: 100%;
    padding: 8px 10px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .nl-input:focus { border-color: var(--accent); }
  .nl-input::placeholder { color: var(--text-disabled); }
  .preview {
    margin-top: 6px;
    font-size: 11px;
    color: var(--accent);
    padding: 0 2px;
  }
  .presets-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-disabled);
    padding: 8px 16px 4px;
    border-top: 1px solid var(--border);
  }
  .option {
    display: block;
    width: 100%;
    padding: 10px 16px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background var(--duration-fast) ease;
  }
  .option:hover { background: var(--bg-hover); }
</style>
