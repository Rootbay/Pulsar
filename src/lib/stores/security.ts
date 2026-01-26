import { appSettings, settingsManager } from './appSettings.svelte';
import type { SecuritySettings } from '../config/settings';

export const securitySettings = {
  subscribe(fn: (value: SecuritySettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.security);
    });
  },
  set(value: SecuritySettings) {
    settingsManager.update((s) => {
      s.security = value;
    });
  },
  update(updater: (s: SecuritySettings) => SecuritySettings) {
    settingsManager.update((s) => {
      s.security = updater(s.security);
    });
  },
  get value() {
    return settingsManager.current.security;
  },
  set value(v: SecuritySettings) {
    settingsManager.update((s) => {
      s.security = v;
    });
  }
};

