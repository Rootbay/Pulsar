import { appSettings, settingsManager } from './appSettings.svelte';
import type { GeneratorSettings } from '../config/settings';

export const generatorSettings = {
  subscribe(fn: (value: GeneratorSettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.generator);
    });
  },
  set(value: GeneratorSettings) {
    settingsManager.update((s) => {
      s.generator = value;
    });
  },
  update(updater: (s: GeneratorSettings) => GeneratorSettings) {
    settingsManager.update((s) => {
      s.generator = updater(s.generator);
    });
  },
  get value() {
    return settingsManager.current.generator;
  },
  set value(v: GeneratorSettings) {
    settingsManager.update((s) => {
      s.generator = v;
    });
  }
};
