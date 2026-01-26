import { writable, derived, type Writable, type Readable, get } from 'svelte/store';
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
  const initialValue = writable<T>(defaultValue);
  const store = writable<T>(defaultValue);

  const hasUnsavedChanges = derived([initialValue, store], ([$initialValue, $store]) => {
    return !deepEqual($initialValue, $store);
  });

  async function load() {
    try {
      const data = await invoke<string | null>(readCommand);
      if (data) {
        const parsed = JSON.parse(data);
        initialValue.set(parsed);
        store.set(parsed);
      }
    } catch (e) {
      console.debug(`[${moduleName}] could not load from DB:`, e);
    }
  }

  async function save() {
    const current = get(store);
    try {
      await invoke(writeCommand, { settingsJson: JSON.stringify(current) });
      initialValue.set(JSON.parse(JSON.stringify(current)));
    } catch (e) {
      console.error(`Failed to save ${moduleName}:`, e);
      throw e;
    }
  }

  async function reset() {
    const initial = get(initialValue);
    store.set(JSON.parse(JSON.stringify(initial)));
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
    subscribe: store.subscribe,
    set: store.set,
    update: store.update,
    save,
    reset,
    load,
    hasUnsavedChanges
  };
}
