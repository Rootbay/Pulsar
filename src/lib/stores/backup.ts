import { appSettings, settingsManager } from './appSettings.svelte';
import type { BackupSettings } from '../config/settings';

export const backupSettings = {
  subscribe(fn: (value: BackupSettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.backup);
    });
  },
  set(value: BackupSettings) {
    settingsManager.update((s) => {
      s.backup = value;
    });
  },
  update(updater: (s: BackupSettings) => BackupSettings) {
    settingsManager.update((s) => {
      s.backup = updater(s.backup);
    });
  },
  get value() {
    return settingsManager.current.backup;
  },
  set value(v: BackupSettings) {
    settingsManager.update((s) => {
      s.backup = v;
    });
  }
};

