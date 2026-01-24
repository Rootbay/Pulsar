import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import type { BackupSettings } from '../config/settings';

function createBackupSettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.backup);

  return {
    subscribe,
    set: (value: BackupSettings) => {
      appSettings.update((settings) => {
        settings.backup = value;
        return settings;
      });
    },
    update: (callback: (settings: BackupSettings) => BackupSettings) => {
      appSettings.update((settings) => {
        settings.backup = callback(settings.backup);
        return settings;
      });
    }
  };
}

export const backupSettings = createBackupSettingsStore();
