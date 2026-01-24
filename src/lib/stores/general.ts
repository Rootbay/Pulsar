import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import type { GeneralSettings } from '../config/settings';

function createGeneralSettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.general);

  return {
    subscribe,
    set: (value: GeneralSettings) => {
      appSettings.update((settings) => {
        settings.general = value;
        return settings;
      });
    }
  };
}

export const generalSettings = createGeneralSettingsStore();
