<script lang="ts">
    import Input from './Input.svelte';
    import Switch from './Switch.svelte';
    import { createEventDispatcher } from 'svelte';

    export let show: boolean;
    export let item: any; // This will be either a preset or a rule
    export let type: 'preset' | 'rule';
    
    const dispatch = createEventDispatcher();

    let editedItem: any;

    $: if (item) {
        editedItem = { ...item };
    }

    function handleSave() {
        dispatch('save', editedItem);
        close();
    }

    function handleCancel() {
        close();
    }

    function close() {
        dispatch('close');
    }
</script>

{#if show}
<div class="overlay" on:click|self={handleCancel}>
  <div class="dialog">
    <button class="close-button" on:click={close} aria-label="Close edit modal">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m18 6-12 12"/>
            <path d="m6 6 12 12"/>
        </svg>
    </button>
    <div class="dialog-header">
        <h2 class="dialog-title">Edit {type === 'preset' ? 'Preset' : 'Rule'}</h2>
    </div>
    <div class="dialog-content">
        {#if type === 'preset'}
            <Input title="Preset Name" bind:inputValue={editedItem.name} />
            <Input title="Length" type="number" bind:inputValue={editedItem.length} />
            <div class="toggle-group">
                <span class="toggle-label">Include uppercase (A-Z)</span>
                <Switch bind:checked={editedItem.settings.uppercase} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Include lowercase (a-z)</span>
                <Switch bind:checked={editedItem.settings.lowercase} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Include digits (0-9)</span>
                <Switch bind:checked={editedItem.settings.digits} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Include symbols (!@#$...)</span>
                <Switch bind:checked={editedItem.settings.symbols} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Avoid ambiguous characters</span>
                <Switch bind:checked={editedItem.settings.ambiguous} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Exclude similar</span>
                <Switch bind:checked={editedItem.settings.similar} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Pronounceable mode</span>
                <Switch bind:checked={editedItem.settings.pronounceable} />
            </div>
        {:else if type === 'rule'}
            <Input title="URL" bind:inputValue={editedItem.url} />
            <Input title="Length" type="number" bind:inputValue={editedItem.length} />
            <Input title="Type" bind:inputValue={editedItem.type} />
            <Input title="Description" bind:inputValue={editedItem.desc} />
            <div class="toggle-group">
                <span class="toggle-label">Include uppercase (A-Z)</span>
                <Switch bind:checked={editedItem.settings.uppercase} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Include lowercase (a-z)</span>
                <Switch bind:checked={editedItem.settings.lowercase} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Include digits (0-9)</span>
                <Switch bind:checked={editedItem.settings.digits} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Include symbols (!@#$...)</span>
                <Switch bind:checked={editedItem.settings.symbols} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Avoid ambiguous characters</span>
                <Switch bind:checked={editedItem.settings.ambiguous} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Exclude similar</span>
                <Switch bind:checked={editedItem.settings.similar} />
            </div>
            <div class="toggle-group">
                <span class="toggle-label">Pronounceable mode</span>
                <Switch bind:checked={editedItem.settings.pronounceable} />
            </div>
        {/if}
    </div>
    <div class="dialog-footer">
        <button class="button" on:click={handleCancel}>Cancel</button>
        <button class="button primary" on:click={handleSave}>Save</button>
    </div>
  </div>
</div>
{/if}

<style>
    .overlay {
        position: fixed;
        inset: 0;
        background: hsl(var(--background) / 0.8);
        backdrop-filter: blur(12px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 50;
    }

    .dialog {
        background: hsl(var(--card));
        border: 1px solid hsl(var(--border));
        border-radius: 16px;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
        width: 90%;
        max-width: 500px;
        max-height: 90vh;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }
    
    .dialog-header {
        padding: 24px 24px 16px 24px;
        border-bottom: 1px solid hsl(var(--border));
    }

    .dialog-title {
        font-size: 20px;
        font-weight: 600;
        color: hsl(var(--foreground));
        margin-bottom: 8px;
    }

    .dialog-content {
        padding: 24px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .dialog-footer {
        padding: 16px 24px 24px 24px;
        border-top: 1px solid hsl(var(--border));
        display: flex;
        justify-content: flex-end;
        gap: 12px;
    }

    .button {
        background: hsl(var(--secondary));
        border: 1px solid hsl(var(--border));
        color: hsl(var(--foreground));
        padding: 8px 16px;
        border-radius: 8px;
        cursor: pointer;
        font-size: 14px;
        font-weight: 500;
        transition: all 0.2s;
        font-family: inherit;
    }

    .button:hover {
        background: hsl(var(--muted));
    }

    .button.primary {
        background: hsl(var(--primary));
        border-color: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
    }

    .button.primary:hover {
        background: hsl(var(--primary) / 0.9);
    }

    .close-button {
        position: absolute;
        top: 16px;
        right: 16px;
        width: 32px;
        height: 32px;
        border-radius: 6px;
        background: transparent;
        border: none;
        color: hsl(var(--muted-foreground));
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
    }

    .close-button:hover {
        background: hsl(var(--muted) / 0.3);
        color: hsl(var(--foreground));
    }
    
    .toggle-group {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
    }

    .toggle-label {
        font-size: 0.875rem;
        color: hsl(var(--foreground));
    }
</style>
