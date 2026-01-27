import { appSettings, settingsManager } from './appSettings.svelte';
import type { AutofillSettings } from '../config/settings';

export const autofillSettings = {
  subscribe(fn: (value: AutofillSettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.autofill);
    });
  },
  set(value: AutofillSettings) {
    settingsManager.update((s) => {
      s.autofill = value;
    });
  },
  update(updater: (s: AutofillSettings) => AutofillSettings) {
    settingsManager.update((s) => {
      s.autofill = updater(s.autofill);
    });
  },
  get value() {
    return settingsManager.current.autofill;
  },
  set value(v: AutofillSettings) {
    settingsManager.update((s) => {
      s.autofill = v;
    });
  }
};
