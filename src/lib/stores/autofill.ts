import { derived } from 'svelte/store';
import { appSettings } from './appSettings.svelte';
import type { AutofillSettings } from '../config/settings';

function createAutofillSettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.autofill);

  return {
    subscribe,
    set: (value: AutofillSettings) => {
      appSettings.update((settings) => {
        settings.autofill = value;
        return settings;
      });
    },
    update: (callback: (settings: AutofillSettings) => AutofillSettings) => {
      appSettings.update((settings) => {
        settings.autofill = callback(settings.autofill);
        return settings;
      });
    }
  };
}

export const autofillSettings = createAutofillSettingsStore();
