<script lang="ts">
  import { createEventDispatcher, tick } from 'svelte';

  export let show: boolean = false;
  const dispatch = createEventDispatcher();

  const passwordManagers = [
    { id: 'lastpass', name: 'LastPass' },
    { id: '1password', name: '1Password' },
    { id: 'keepass', name: 'KeePass' },
    { id: 'bitwarden', name: 'Bitwarden' },
  ];

  let selectedManager: string | null = null;
  let dialogEl: HTMLElement | null = null;

  function handleImport() {
    if (selectedManager) {
      dispatch('importSelected', { manager: selectedManager });
      show = false;
    }
  }

  function handleClose() {
    show = false;
  }

  function onBackdropKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }

  function onDialogKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }

  $: if (show) {
    tick().then(() => {
      dialogEl?.focus();
    });
  }
</script>

{#if show}
  <div
    class="popup-backdrop"
    role="presentation"
    on:click={handleClose}
    on:keydown={onBackdropKeydown}
  >
    <div
      class="popup-content"
      role="dialog"
      aria-modal="true"
      aria-label="Import from another Password Manager"
      tabindex="-1"
      bind:this={dialogEl}
      on:click|stopPropagation
      on:keydown={onDialogKeydown}
    >
      <h3>Import from another Password Manager</h3>

      <div class="manager-list">
        {#each passwordManagers as manager (manager.id)}
          <button
            class="manager-item"
            class:selected={selectedManager === manager.id}
            on:click={() => (selectedManager = manager.id)}
            type="button"
          >
            {manager.name}
          </button>
        {/each}
      </div>

      <div class="popup-actions">
        <button on:click={handleImport} disabled={!selectedManager} type="button">Import</button>
        <button on:click={handleClose} type="button">Cancel</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .popup-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 200;
  }

  .popup-content {
    background-color: #2a2a30;
    padding: 25px;
    border-radius: 10px;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.3);
    width: 90%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    gap: 20px;
    color: var(--white);
    outline: none;
  }

  h3 {
    font-size: 22px;
    text-align: center;
    margin-bottom: 10px;
  }

  .manager-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .manager-item {
    background-color: #3a3a47;
    color: var(--white);
    padding: 12px 15px;
    border-radius: 8px;
    border: 1px solid #4a4a57;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.2s ease, border-color 0.2s ease;
  }

  .manager-item:hover {
    background-color: #4a4a57;
  }

  .manager-item.selected {
    background-color: #5a5a67;
    border-color: #7b7b87;
  }

  .popup-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 15px;
  }

  .popup-actions button {
    padding: 10px 20px;
    border-radius: 20px;
    border: none;
    cursor: pointer;
    font-size: 16px;
    transition: background-color 0.2s ease;
  }

  .popup-actions button:first-child {
    background-color: #007bff;
    color: var(--white);
  }

  .popup-actions button:first-child:hover:not(:disabled) {
    background-color: #0056b3;
  }

  .popup-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .popup-actions button:last-child {
    background-color: #6c757d;
    color: var(--white);
  }

  .popup-actions button:last-child:hover {
    background-color: #5a6268;
  }
</style>
