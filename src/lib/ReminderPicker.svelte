<script lang="ts">
  import { reminderPickerOpen, activeFilePath } from './stores';
  import { setReminderAndArchive } from './actions';
  import { timeUntil } from './utils';

  interface ReminderOption {
    label: string;
    getTime: () => string;
  }

  function isoString(date: Date): string {
    return date.toISOString().split('.')[0];
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

  let nlInput = '';
  let inputEl: HTMLInputElement;

  $: if ($reminderPickerOpen) {
    nlInput = '';
    setTimeout(() => inputEl?.focus(), 50);
  }

  function offsetFromNow(n: number, unit: string): Date {
    const d = new Date();
    if (unit === 'm') d.setMinutes(d.getMinutes() + n);
    else if (unit === 'h') d.setHours(d.getHours() + n);
    else if (unit === 'd') d.setDate(d.getDate() + n);
    else if (unit === 'w') d.setDate(d.getDate() + n * 7);
    return d;
  }

  const UNIT_PATTERN = '(m|min|mins|minutes?|h|hr|hrs|hours?|d|days?|w|wk|wks|weeks?)';
  const relativeRe = new RegExp(`^(?:in\\s+)?(\\d+)\\s*${UNIT_PATTERN}$`);

  function parseNaturalTime(input: string): Date | null {
    const s = input.trim().toLowerCase();
    if (!s) return null;

    // Relative: "30m", "2h", "1d", "in 2 hours", etc.
    const relMatch = s.match(relativeRe);
    if (relMatch) {
      return offsetFromNow(parseInt(relMatch[1]), relMatch[2][0]);
    }

    // Named times
    if (s === 'tonight') {
      const d = new Date();
      d.setHours(20, 0, 0, 0);
      if (d.getTime() < Date.now()) d.setDate(d.getDate() + 1);
      return d;
    }
    if (s === 'tomorrow' || s === 'tomorrow morning' || s === 'tmr' || s === 'tmrw') {
      const d = new Date();
      d.setDate(d.getDate() + 1);
      d.setHours(9, 0, 0, 0);
      return d;
    }
    if (s === 'tomorrow afternoon' || s === 'tmr afternoon') {
      const d = new Date();
      d.setDate(d.getDate() + 1);
      d.setHours(14, 0, 0, 0);
      return d;
    }
    if (s === 'tomorrow evening' || s === 'tmr evening' || s === 'tomorrow night') {
      const d = new Date();
      d.setDate(d.getDate() + 1);
      d.setHours(20, 0, 0, 0);
      return d;
    }
    if (s === 'next week') {
      const d = new Date();
      d.setDate(d.getDate() + 7);
      d.setHours(9, 0, 0, 0);
      return d;
    }
    if (s === 'next month') {
      const d = new Date();
      d.setMonth(d.getMonth() + 1);
      d.setHours(9, 0, 0, 0);
      return d;
    }

    // Day names: "monday", "tuesday", etc. → next occurrence at 9 AM
    const days = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'];
    const dayIdx = days.indexOf(s);
    if (dayIdx !== -1) {
      const d = new Date();
      const today = d.getDay();
      let diff = dayIdx - today;
      if (diff <= 0) diff += 7;
      d.setDate(d.getDate() + diff);
      d.setHours(9, 0, 0, 0);
      return d;
    }

    return null;
  }

  $: parsed = parseNaturalTime(nlInput);
  $: preview = parsed ? formatPreview(parsed) : '';

  function formatPreview(d: Date): string {
    const relative = timeUntil(d.toISOString());
    const timeStr = d.toLocaleString('en-US', {
      weekday: 'short', month: 'short', day: 'numeric',
      hour: 'numeric', minute: '2-digit',
    });
    return `${timeStr} (${relative})`;
  }

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
      setReminder(isoString(parsed));
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
  <div class="backdrop" on:click={close} on:keydown={handleKeydown} role="button" tabindex="-1">
    <div class="picker" on:click|stopPropagation on:keydown={() => {}} role="dialog" tabindex="-1">
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
      {#each options as opt}
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
  .nl-input-wrap {
    padding: 10px 12px 6px;
  }
  .nl-input {
    width: 100%;
    padding: 8px 10px;
    background: #1a1a1a;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 13px;
    outline: none;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .nl-input:focus { border-color: #5b9bd5; }
  .nl-input::placeholder { color: #555; }
  .preview {
    margin-top: 6px;
    font-size: 11px;
    color: #5b9bd5;
    padding: 0 2px;
  }
  .presets-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #555;
    padding: 8px 16px 4px;
    border-top: 1px solid #3a3a3a;
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
</style>
