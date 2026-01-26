import { appSettings, settingsManager } from './appSettings.svelte';
import { defaultAppearanceSettings, type AppearanceSettings } from '../config/settings';

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
  get hasUnsavedChanges() {
    // Using a getter with Svelte 5 reactivity
    const current = settingsManager.current.appearance;
    // We would need a way to track the initial state. 
    // Since settingsManager initializes from backend, we can capture the first loaded state there.
    // For now, let's keep it simple or implement a proper comparison if needed.
    return false; // Placeholder for improved logic if initial state is tracked
  }
};

export type AppearanceSettingsStore = typeof appearanceSettings;

