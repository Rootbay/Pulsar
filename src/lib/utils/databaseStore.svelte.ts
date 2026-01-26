import { type Writable, type Readable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import deepEqual from 'fast-deep-equal';
import { settingsStore } from '../stores/settingsStore';

interface DatabaseSettingsModule<T> extends Writable<T> {
  save: () => Promise<void>;
  reset: () => Promise<void>;
  load: () => Promise<void>;
  hasUnsavedChanges: Readable<boolean>;
}

export function createDatabaseStore<T>(
  moduleName: string,
  defaultValue: T,
  readCommand: string,
  writeCommand: string
): DatabaseSettingsModule<T> {
  let initialValue = $state<T>(defaultValue);
  let currentValue = $state<T>(defaultValue);

  // Custom subscribe for hasUnsavedChanges to match Readable<boolean>
  const hasUnsavedChanges: Readable<boolean> = {
    subscribe(fn) {
      const effect = $effect.root(() => {
        $effect(() => {
          const isDirty = !deepEqual(initialValue, currentValue);
          fn(isDirty);
        });
      });
      return () => effect();
    }
  };

  async function load() {
    try {
      const data = await invoke<string | null>(readCommand);
      if (data) {
        const parsed = JSON.parse(data);
        // Use structuredClone or JSON parse/stringify to break references if needed,
        // but for now simple assignment if T is simple object.
        // Ensuring deep copy for initialValue is safer.
        initialValue = JSON.parse(JSON.stringify(parsed));
        currentValue = JSON.parse(JSON.stringify(parsed));
      }
    } catch (e) {
      console.debug(`[${moduleName}] could not load from DB:`, e);
    }
  }

  async function save() {
    try {
      // Snapshot current value
      const snapshot = $state.snapshot(currentValue);
      await invoke(writeCommand, { settingsJson: JSON.stringify(snapshot) });
      initialValue = JSON.parse(JSON.stringify(snapshot));
    } catch (e) {
      console.error(`Failed to save ${moduleName}:`, e);
      throw e;
    }
  }

  async function reset() {
    // Revert to initialValue
    currentValue = JSON.parse(JSON.stringify(initialValue));
  }

  settingsStore.registerModule(moduleName, {
    save: () => {
      save().catch(console.error);
    },
    reset: () => {
      reset().catch(console.error);
    },
    hasUnsavedChanges
  });

  return {
    subscribe(fn) {
      const effect = $effect.root(() => {
        $effect(() => {
          fn(currentValue);
        });
      });
      return () => effect();
    },
    set(value: T) {
      currentValue = value;
    },
    update(updater: (value: T) => T) {
      currentValue = updater(currentValue);
    },
    save,
    reset,
    load,
    hasUnsavedChanges
  };
}

