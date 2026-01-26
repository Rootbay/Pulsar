import { appSettings, settingsManager } from './appSettings.svelte';
import type { GeneralSettings } from '../config/settings';

export const generalSettings = {
  subscribe(fn: (value: GeneralSettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.general);
    });
  },
  set(value: GeneralSettings) {
    settingsManager.update((s) => {
      s.general = value;
    });
  },
  update(updater: (s: GeneralSettings) => GeneralSettings) {
    settingsManager.update((s) => {
      s.general = updater(s.general);
    });
  },
  get value() {
    return settingsManager.current.general;
  },
  set value(v: GeneralSettings) {
    settingsManager.update((s) => {
      s.general = v;
    });
  }
};

