import { appSettings, settingsManager } from './appSettings.svelte';
import type { AdvancedSettings } from '../config/settings';

export const advancedSettings = {
  subscribe(fn: (value: AdvancedSettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.advanced);
    });
  },
  set(value: AdvancedSettings) {
    settingsManager.update((s) => {
      s.advanced = value;
    });
  },
  update(updater: (s: AdvancedSettings) => AdvancedSettings) {
    settingsManager.update((s) => {
      s.advanced = updater(s.advanced);
    });
  },
  get value() {
    return settingsManager.current.advanced;
  },
  set value(v: AdvancedSettings) {
    settingsManager.update((s) => {
      s.advanced = v;
    });
  }
};

