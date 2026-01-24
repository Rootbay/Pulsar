import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import type { SecuritySettings } from '../config/settings';

function createSecuritySettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.security);

  return {
    subscribe,
    set: (value: SecuritySettings) => {
      appSettings.update((settings) => {
        settings.security = value;
        return settings;
      });
    }
  };
}

export const securitySettings = createSecuritySettingsStore();
