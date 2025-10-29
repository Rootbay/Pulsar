import { browser } from '$app/environment';
import { writable, type Writable } from 'svelte/store';

type PersistentStore<T> = Writable<T> & {
  reset: () => void;
};

function readInitialValue<T>(key: string, fallback: T): T {
  if (!browser) {
    return fallback;
  }

  try {
    const storedValue = localStorage.getItem(key);
    if (storedValue === null) {
      return fallback;
    }

    return JSON.parse(storedValue) as T;
  } catch (error) {
    console.warn(`Failed to parse value for persistent store "${key}"`, error);
    localStorage.removeItem(key);
    return fallback;
  }
}

function writeValue<T>(key: string, value: T, fallback: T) {
  if (!browser) {
    return;
  }

  try {
    if (value === fallback) {
      localStorage.removeItem(key);
    } else {
      localStorage.setItem(key, JSON.stringify(value));
    }
  } catch (error) {
    console.warn(`Failed to persist value for store "${key}"`, error);
  }
}

export function persistentWritable<T>(key: string, initialValue: T): PersistentStore<T> {
  const startValue = readInitialValue<T>(key, initialValue);
  const store = writable<T>(startValue);

  return {
    subscribe: store.subscribe,
    set(value: T) {
      writeValue(key, value, initialValue);
      store.set(value);
    },
    update(updater) {
      store.update((current) => {
        const nextValue = updater(current);
        writeValue(key, nextValue, initialValue);
        return nextValue;
      });
    },
    reset() {
      writeValue(key, initialValue, initialValue);
      store.set(initialValue);
    }
  } satisfies PersistentStore<T>;
}
