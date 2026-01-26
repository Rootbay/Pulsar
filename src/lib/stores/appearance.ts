import { appSettings, settingsManager } from './appSettings.svelte';
import { defaultAppearanceSettings, type AppearanceSettings } from '../config/settings';

const hasUnsavedChanges = {
  subscribe(fn: (changed: boolean) => void, invalidate?: (value?: boolean) => void) {
    let initialSettings: string | undefined;
    return appSettings.subscribe((settings) => {
      const current = settings.appearance;
      const currentStr = JSON.stringify(current);
      if (initialSettings === undefined) {
        initialSettings = currentStr;
      }
      fn(currentStr !== initialSettings);
    });
  }
};

export const appearanceSettings = {
  subscribe(fn: (value: AppearanceSettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.appearance);
    });
  },
  get value() {
    return settingsManager.current.appearance;
  },
  set(value: AppearanceSettings) {
    settingsManager.update((s) => {
      s.appearance = value;
    });
  },
  update(updater: (s: AppearanceSettings) => AppearanceSettings) {
    settingsManager.update((s) => {
      s.appearance = updater(s.appearance);
    });
  },
  save: () => {
    // Trigger save
    settingsManager.update((s) => s);
  },
  reset: () => {
    settingsManager.update((settings) => {
      settings.appearance = defaultAppearanceSettings;
    });
  },
  hasUnsavedChanges
};

export type AppearanceSettingsStore = typeof appearanceSettings;

