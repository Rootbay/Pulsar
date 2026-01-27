import { settings } from './appSettings.svelte';
import type { GeneralSettings } from '../config/settings';

class GeneralSettingsStore {
  get state() {
    return settings.state.general;
  }

  update<K extends keyof GeneralSettings>(key: K, value: GeneralSettings[K]) {
    settings.state.general[key] = value;
    settings.save();
  }

  toggle(
    key: {
      [K in keyof GeneralSettings]: GeneralSettings[K] extends boolean ? K : never;
    }[keyof GeneralSettings]
  ) {
    this.update(key, !this.state[key]);
  }
}

export const generalSettings = new GeneralSettingsStore();
