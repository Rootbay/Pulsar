import { writable, derived } from 'svelte/store';
import * as fastDeepEqual from 'fast-deep-equal';
const deepEqual = fastDeepEqual.default;
import { appSettings, settingsManager } from './appSettings.svelte';
import type { GeneralSettings } from '../config/settings';

const baselineStore = writable<GeneralSettings | null>(null);

appSettings.subscribe((settings) => {
  if (settings && settings.general) {
    baselineStore.update((curr) => {
      if (curr === null) return structuredClone(settings.general);
      return curr;
    });
  }
});

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
  },
  save: () => {
    baselineStore.set(structuredClone(settingsManager.current.general));
  },
  reset: () => {
    baselineStore.subscribe((val) => {
      if (val) {
        settingsManager.update((s) => {
          s.general = structuredClone(val);
        });
      }
    })();
  },
  hasUnsavedChanges: derived([appSettings, baselineStore], ([$appSettings, $baseline]) => {
    if (!$baseline || !$appSettings?.general) return false;
    return !deepEqual($appSettings.general, $baseline);
  })
};
