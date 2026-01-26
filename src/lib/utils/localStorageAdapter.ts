import { writable, derived, type Writable, type Readable, get } from 'svelte/store';
import * as fastDeepEqual from 'fast-deep-equal';
const deepEqual = fastDeepEqual.default;
import { settingsStore } from '../stores/settingsStore';
import { browser } from '$app/environment';

interface SettingsModule<T> extends Writable<T> {
  save: () => void;
  reset: () => void;
  hasUnsavedChanges: Readable<boolean>;
}

function cloneValue<T>(value: T): T {
  if (value && typeof value === 'object') {
    try {
      return JSON.parse(JSON.stringify(value)) as T;
    } catch {
      return value;
    }
  }
  return value;
}

function createLocalStorageStore<T>(key: string, startValue: T): SettingsModule<T> {
  const initialValue = writable<T>(startValue);
  const settings = writable<T>(startValue);

  if (browser) {
    const storedValue = localStorage.getItem(key);
    if (storedValue) {
      try {
        const parsedValue = JSON.parse(storedValue) as T;
        initialValue.set(parsedValue);
        settings.set(parsedValue);
      } catch (error) {
        console.warn(`Failed to parse local storage value for "${key}"`, error);
        localStorage.removeItem(key);
      }
    }
  }

  const hasUnsavedChanges = derived([initialValue, settings], ([$initialValue, $settings]) => {
    return !deepEqual($initialValue, $settings);
  });

  function save() {
    const currentSettings = get(settings);
    initialValue.set(cloneValue(currentSettings));
    if (browser) {
      localStorage.setItem(key, JSON.stringify(currentSettings));
    }
  }

  function reset() {
    const currentInitialSettings = get(initialValue);
    settings.set(cloneValue(currentInitialSettings));
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
