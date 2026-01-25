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
      let count = 0;
      const unsubscribes = $unsavedStores.map((store) =>
        store.subscribe((value) => {
          if (value) {
            count++;
          } else {
            count--;
          }
          set(count > 0);
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
