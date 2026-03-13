<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { toasts } from './stores';
  import { dismissToast } from './actions';

  function handleUndo(toast: { id: number; undoAction: (() => Promise<void>) | null }) {
    if (toast.undoAction) {
      toast.undoAction();
    }
    dismissToast(toast.id);
  }
</script>

{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as toast (toast.id)}
      <div
        class="toast"
        transition:fly={{ y: 20, duration: 200 }}
        role="status"
      >
        <span class="toast-message">{toast.message}</span>
        {#if toast.undoAction}
          <button class="undo-btn" on:click={() => handleUndo(toast)}>Undo</button>
        {/if}
        <button class="close-btn" on:click={() => dismissToast(toast.id)}>&times;</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 200;
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--bg-active);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 16px;
    color: var(--text-primary);
    font-size: 13px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    pointer-events: auto;
    white-space: nowrap;
  }
  .toast-message {
    flex: 1;
  }
  .undo-btn {
    background: transparent;
    border: none;
    color: var(--accent);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    padding: 0;
  }
  .undo-btn:hover {
    text-decoration: underline;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-disabled);
    font-size: 16px;
    cursor: pointer;
    padding: 0 2px;
    line-height: 1;
  }
  .close-btn:hover {
    color: var(--text-primary);
  }
</style>
