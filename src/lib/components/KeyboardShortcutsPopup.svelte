<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { keybinds } from '../stores/keybinds';
  import { writable, type Writable } from 'svelte/store';
  import type { Keybind } from '../config/keybinds';

  const dispatch = createEventDispatcher();

  let duplicateKeybinds: Writable<{[key: string]: boolean}> = writable({}); // { keybindName: true/false }
  let shakeInput: Writable<{[key: string]: boolean}> = writable({}); // { keybindName: true/false }

  function close() {
    dispatch('close');
  }

  function validateKeybinds() {
    const currentKeybinds = $keybinds;
    const keyMap = new Map<string, string[]>();
    let hasDuplicates = false;
    let newDuplicateState: {[key: string]: boolean} = {};

    currentKeybinds.forEach((kb: Keybind) => {
      if (keyMap.has(kb.key)) {
        keyMap.get(kb.key)!.push(kb.name);
      } else {
        keyMap.set(kb.key, [kb.name]);
      }
    });

    keyMap.forEach((names: string[], key: string) => {
      if (names.length > 1) {
        hasDuplicates = true;
        names.forEach((name: string) => {
          newDuplicateState[name] = true;
        });
      }
    });

    duplicateKeybinds.set(newDuplicateState);
    return !hasDuplicates;
  }

  function handleKeydown(event: KeyboardEvent, keybindName: string) {
    event.preventDefault();
    let keys = [];
    if (event.ctrlKey) keys.push('Ctrl');
    if (event.shiftKey) keys.push('Shift');
    if (event.altKey) keys.push('Alt');
    if (event.metaKey) keys.push('Meta'); // For Cmd on Mac

    // Add the actual key, unless it's a modifier key already added
    if (!['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) {
      keys.push(event.key.length === 1 ? event.key.toUpperCase() : event.key);
    }

    const newKey = keys.join('+');
    keybinds.updateKeybind(keybindName, newKey);
    validateKeybinds(); // Validate after update
  }

  function handleReset() {
    keybinds.resetKeybinds();
    validateKeybinds(); // Validate after reset
  }

  function handleSave() {
    if (validateKeybinds()) {
      // No duplicates, proceed with saving (or closing)
      close();
    } else {
      // Duplicates exist, trigger shake animation for all duplicate inputs
      let newShakeState: {[key: string]: boolean} = {};
      for (const name in $duplicateKeybinds) {
        if ($duplicateKeybinds[name]) {
          newShakeState[name] = true;
        }
      }
      shakeInput.set(newShakeState);

      // Remove shake class after animation
      setTimeout(() => {
        shakeInput.set({});
      }, 500); // Match animation duration
    }
  }

  // Initial validation when component mounts
  validateKeybinds();
</script>

<div class="overlay">
  <div class="dialog">
    <button class="close-button" on:click={close} aria-label="Close keyboard shortcuts popup">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m18 6-12 12"/>
        <path d="m6 6 12 12"/>
      </svg>
    </button>

    <div class="dialog-header">
      <h2 class="dialog-title">Keyboard Shortcuts</h2>
      <p class="dialog-description">
        Customize your keyboard shortcuts for faster password management.
      </p>
    </div>

    <div class="dialog-content">
      <div class="shortcuts-list">
        {#each $keybinds as keybind (keybind.name)}
          <div class="shortcut-item">
            <div class="shortcut-info">
              <div class="shortcut-action">{keybind.name}</div>
              <div class="shortcut-scope">{keybind.description}</div>
            </div>
            <input
              type="text"
              class="shortcut-input"
              class:duplicate-input={$duplicateKeybinds[keybind.name]}
              class:shake={$shakeInput[keybind.name]}
              bind:value={keybind.key}
              on:keydown={(event) => handleKeydown(event, keybind.name)}
              readonly
            />
          </div>
        {/each}
      </div>
    </div>

    <div class="dialog-footer">
      <button class="button" on:click={handleReset}>Reset to Default</button>
      <button class="button primary" on:click={handleSave}>Save Changes</button>
    </div>
  </div>
</div>

<style>
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }

    .overlay {
        position: fixed;
        inset: 0;
        background: hsl(var(--background) / 0.8);
        backdrop-filter: blur(12px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 50;
        animation: fadeIn 0.3s ease-out;
    }

    .dialog {
        background: hsl(var(--card));
        border: 1px solid hsl(var(--border));
        border-radius: 16px;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
        width: 90%;
        max-width: 672px;
        max-height: 90vh;
        overflow: hidden;
        animation: scaleIn 0.3s ease-out;
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

    .dialog-description {
        font-size: 14px;
        color: hsl(var(--muted-foreground));
    }

    .dialog-content {
        padding: 24px;
        max-height: 384px;
        overflow-y: auto;
    }

    .shortcuts-list {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .shortcut-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px;
        border: 1px solid hsl(var(--border));
        border-radius: 8px;
        transition: all 0.2s;
    }

    .shortcut-item:hover {
        background: hsl(var(--muted) / 0.3);
        border-color: hsl(var(--primary) / 0.3);
    }

    .shortcut-info {
        flex: 1;
    }

    .shortcut-action {
        font-weight: 500;
        color: hsl(var(--foreground));
        margin-bottom: 4px;
    }

    .shortcut-scope {
        font-size: 12px;
        color: hsl(var(--muted-foreground));
    }

    .shortcut-input {
        background: hsl(var(--background) / 0.5);
        border: 1px solid hsl(var(--border));
        color: hsl(var(--foreground));
        padding: 8px 12px;
        border-radius: 6px;
        font-size: 14px;
        width: 160px;
        text-align: center;
        font-family: monospace;
        transition: all 0.2s;
    }

    .shortcut-input:focus {
        outline: none;
        border-color: hsl(var(--primary));
        box-shadow: 0 0 0 2px hsl(var(--primary) / 0.2);
    }

    .shortcut-input.duplicate-input {
        border-color: hsl(var(--destructive));
        box-shadow: 0 0 0 2px hsl(var(--destructive) / 0.2);
    }

    .shortcut-input.shake {
        animation: shake 0.5s cubic-bezier(.36,.07,.19,.97) both;
        transform: translate3d(0, 0, 0);
        backface-visibility: hidden;
        perspective: 1000px;
    }

    @keyframes shake {
        10%, 90% {
            transform: translate3d(-1px, 0, 0);
        }
        
        20%, 80% {
            transform: translate3d(2px, 0, 0);
        }

        30%, 50%, 70% {
            transform: translate3d(-4px, 0, 0);
        }

        40%, 60% {
            transform: translate3d(4px, 0, 0);
        }
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

    .dialog-content::-webkit-scrollbar {
        width: 6px;
    }

    .dialog-content::-webkit-scrollbar-track {
        background: hsl(var(--muted) / 0.3);
        border-radius: 3px;
    }

    .dialog-content::-webkit-scrollbar-thumb {
        background: hsl(var(--muted-foreground) / 0.3);
        border-radius: 3px;
    }

    .dialog-content::-webkit-scrollbar-thumb:hover {
        background: hsl(var(--muted-foreground) / 0.5);
    }

    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    @keyframes scaleIn {
        from { transform: scale(0.95); opacity: 0; }
        to { transform: scale(1); opacity: 1; }
    }
</style>