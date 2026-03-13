<script lang="ts">
  import { reminderPickerOpen, activeFilePath } from './stores';
  import { refreshItems } from './actions';
  import { invoke } from '@tauri-apps/api/core';

  interface ReminderOption {
    label: string;
    getTime: () => string;
  }

  function isoString(date: Date): string {
    return date.toISOString().replace('Z', '').split('.')[0];
  }

  const options: ReminderOption[] = [
    {
      label: 'Tonight (8 PM)',
      getTime: () => {
        const d = new Date();
        d.setHours(20, 0, 0, 0);
        if (d.getTime() < Date.now()) d.setDate(d.getDate() + 1);
        return isoString(d);
      },
    },
    {
      label: 'Tomorrow (9 AM)',
      getTime: () => {
        const d = new Date();
        d.setDate(d.getDate() + 1);
        d.setHours(9, 0, 0, 0);
        return isoString(d);
      },
    },
    {
      label: 'In 3 days',
      getTime: () => {
        const d = new Date();
        d.setDate(d.getDate() + 3);
        d.setHours(9, 0, 0, 0);
        return isoString(d);
      },
    },
    {
      label: 'Next week',
      getTime: () => {
        const d = new Date();
        d.setDate(d.getDate() + 7);
        d.setHours(9, 0, 0, 0);
        return isoString(d);
      },
    },
  ];

  let customDate = '';

  async function setReminder(time: string) {
    if (!$activeFilePath) return;
    try {
      await invoke('set_reminder', { path: $activeFilePath, time });
      await refreshItems();
    } catch (e) {
      console.error('Failed to set reminder:', e);
    }
    close();
  }

  function setCustomReminder() {
    if (customDate) {
      setReminder(new Date(customDate).toISOString().split('.')[0]);
    }
  }

  function close() {
    reminderPickerOpen.set(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

{#if $reminderPickerOpen}
  <div class="backdrop" on:click={close} on:keydown={handleKeydown} role="button" tabindex="-1">
    <div class="picker" on:click|stopPropagation on:keydown={() => {}} role="dialog" tabindex="-1">
      <div class="header">Set Reminder</div>
      {#each options as opt}
        <button class="option" on:click={() => setReminder(opt.getTime())}>
          {opt.label}
        </button>
      {/each}
      <div class="custom">
        <input
          type="datetime-local"
          bind:value={customDate}
          class="date-input"
        />
        <button class="option custom-btn" on:click={setCustomReminder} disabled={!customDate}>
          Set custom
        </button>
      </div>
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
    width: 280px;
    background: #252525;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    height: fit-content;
  }
  .header {
    padding: 12px 16px;
    font-size: 13px;
    font-weight: 600;
    color: #e0e0e0;
    border-bottom: 1px solid #3a3a3a;
  }
  .option {
    display: block;
    width: 100%;
    padding: 10px 16px;
    background: transparent;
    border: none;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }
  .option:hover { background: #333; }
  .option:disabled { color: #555; cursor: default; }
  .custom {
    border-top: 1px solid #3a3a3a;
    padding: 8px 16px;
  }
  .date-input {
    width: 100%;
    padding: 6px 8px;
    background: #1a1a1a;
    border: 1px solid #3a3a3a;
    border-radius: 4px;
    color: #ccc;
    font-size: 12px;
    margin-bottom: 4px;
    color-scheme: dark;
  }
  .custom-btn { padding: 8px 16px; }
</style>
