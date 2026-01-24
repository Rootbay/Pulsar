import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import type { AdvancedSettings } from '../config/settings';

function createAdvancedSettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.advanced);

  return {
    subscribe,
    set: (value: AdvancedSettings) => {
      appSettings.update((settings) => {
        settings.advanced = value;
        return settings;
      });
    },
    update: (callback: (settings: AdvancedSettings) => AdvancedSettings) => {
      appSettings.update((settings) => {
        settings.advanced = callback(settings.advanced);
        return settings;
      });
    }
  };
}

export const advancedSettings = createAdvancedSettingsStore();
