import { derived, readable } from 'svelte/store';
import { appSettings } from './appSettings.svelte';
import { defaultAppearanceSettings, type AppearanceSettings } from '../config/settings';

function createAppearanceSettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.appearance);

  const hasUnsavedChanges = readable(false, (setReadable) => {
    let initialSettings: AppearanceSettings;
    const unsubscribe = appSettings.subscribe(($appSettings) => {
      const currentSettings = $appSettings.appearance;
      if (!initialSettings) {
        initialSettings = currentSettings;
      }
      setReadable(JSON.stringify(currentSettings) !== JSON.stringify(initialSettings));
    });
    return unsubscribe;
  });

  return {
    subscribe,
    set: (value: AppearanceSettings) => {
      appSettings.update((settings) => {
        settings.appearance = value;
        return settings;
      });
    },
    update: (callback: (settings: AppearanceSettings) => AppearanceSettings) => {
      appSettings.update((settings) => {
        settings.appearance = callback(settings.appearance);
        return settings;
      });
    },
    save: () => {
      appSettings.update((s) => s); // Dummy update to trigger appSettings save
    },
    reset: () => {
      appSettings.update((settings) => {
        settings.appearance = defaultAppearanceSettings;
        return settings;
      });
    },
    hasUnsavedChanges
  };
}

export type AppearanceSettingsStore = ReturnType<typeof createAppearanceSettingsStore>;

export const appearanceSettings = createAppearanceSettingsStore();
