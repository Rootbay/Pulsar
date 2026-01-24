import { writable, derived, type Writable, type Readable } from 'svelte/store';
import * as fastDeepEqual from 'fast-deep-equal';
const deepEqual = fastDeepEqual.default;
import { settingsStore } from '../stores/settingsStore';
import { browser } from '$app/environment';

interface SettingsModule<T> extends Writable<T> {
  save: () => void;
  reset: () => void;
  hasUnsavedChanges: Readable<boolean>;
}

function createLocalStorageStore<T>(key: string, startValue: T): SettingsModule<T> {
  const initialValue = writable<T>(startValue);
  const settings = writable<T>(startValue);

  if (browser) {
    const storedValue = localStorage.getItem(key);
    if (storedValue) {
      const parsedValue = JSON.parse(storedValue);
      initialValue.set(parsedValue);
      settings.set(parsedValue);
    }
  }

  const hasUnsavedChanges = derived([initialValue, settings], ([$initialValue, $settings]) => {
    return !deepEqual($initialValue, $settings);
  });

  function save() {
    settings.subscribe((currentSettings) => {
      initialValue.set({ ...currentSettings });
      if (browser) {
        localStorage.setItem(key, JSON.stringify(currentSettings));
      }
    })();
  }

  function reset() {
    initialValue.subscribe((currentInitialSettings) => {
      settings.set({ ...currentInitialSettings });
    })();
  }

  settingsStore.registerModule(key, {
    save,
    reset,
    hasUnsavedChanges
  });

  return {
    subscribe: settings.subscribe,
    set: (value: T) => {
      if (browser) {
        localStorage.setItem(key, JSON.stringify(value));
      }
      settings.set(value);
    },
    update: settings.update,
    save,
    reset,
    hasUnsavedChanges
  };
}

export const localStorageAdapter = {
  createStore: createLocalStorageStore
};
