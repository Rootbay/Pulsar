import { writable, derived } from 'svelte/store';
import * as fastDeepEqual from 'fast-deep-equal';
const deepEqual = fastDeepEqual.default;
import { appSettings, settingsManager } from './appSettings.svelte';
import { defaultAppearanceSettings, type AppearanceSettings } from '../config/settings';

const baselineStore = writable<AppearanceSettings | null>(null);

appSettings.subscribe((settings) => {
  if (settings && settings.appearance) {
    baselineStore.update((curr) => {
      if (curr === null) return structuredClone(settings.appearance);
      return curr;
    });
  }
});

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
    baselineStore.set(structuredClone(settingsManager.current.appearance));
  },
  reset: () => {
    baselineStore.subscribe((val) => {
      if (val) {
        settingsManager.update((s) => {
          s.appearance = structuredClone(val);
        });
      }
    })();
  },
  hasUnsavedChanges: derived([appSettings, baselineStore], ([$appSettings, $baseline]) => {
    if (!$baseline || !$appSettings?.appearance) return false;
    return !deepEqual($appSettings.appearance, $baseline);
  })
};

export type AppearanceSettingsStore = typeof appearanceSettings;
