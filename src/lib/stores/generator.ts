import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import type { GeneratorSettings } from '../config/settings';

function createGeneratorSettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.generator);

  return {
    subscribe,
    set: (value: GeneratorSettings) => {
      appSettings.update((settings) => {
        settings.generator = value;
        return settings;
      });
    },
    update: (callback: (settings: GeneratorSettings) => GeneratorSettings) => {
      appSettings.update((settings) => {
        settings.generator = callback(settings.generator);
        return settings;
      });
    }
  };
}

export const generatorSettings = createGeneratorSettingsStore();
