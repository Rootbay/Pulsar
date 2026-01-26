import { writable, derived, type Readable } from 'svelte/store';

interface SettingsModule {
  save: () => void;
  reset: () => void;
  hasUnsavedChanges: Readable<boolean>;
}

const registeredModules = writable<Map<string, SettingsModule>>(new Map());

function createSettingsStore() {
  const allUnsavedChanges = derived(registeredModules, ($modules) => {
    const unsavedStores: Readable<boolean>[] = [];
    for (const module of Array.from($modules.values())) {
      unsavedStores.push(module.hasUnsavedChanges);
    }
    return unsavedStores;
  });

  const hasAnyUnsavedChanges = derived(
    allUnsavedChanges,
    ($unsavedStores, set) => {
      const states = new Map<number, boolean>();
      const computeAny = () => {
        for (const value of states.values()) {
          if (value) return true;
        }
        return false;
      };

      const unsubscribes = $unsavedStores.map((store, index) =>
        store.subscribe((value) => {
          states.set(index, value);
          set(computeAny());
        })
      );

      return () => unsubscribes.forEach((unsub) => unsub());
    },
    false
  );

  function registerModule(name: string, module: SettingsModule) {
    registeredModules.update((modules) => {
      modules.set(name, module);
      return modules;
    });
  }

  function saveAll() {
    registeredModules.update((modules) => {
      for (const module of Array.from(modules.values())) {
        module.save();
      }
      return modules;
    });
  }

  function resetAll() {
    registeredModules.update((modules) => {
      for (const module of Array.from(modules.values())) {
        module.reset();
      }
      return modules;
    });
  }

  return {
    subscribe: hasAnyUnsavedChanges.subscribe,
    registerModule,
    saveAll,
    resetAll
  };
}

export const settingsStore = createSettingsStore();
