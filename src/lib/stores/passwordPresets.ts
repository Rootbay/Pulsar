import { appSettings, settingsManager } from './appSettings.svelte';
import { type PasswordPreset, defaultPasswordPresets } from '../config/settings';

export const passwordPresets = {
  subscribe(fn: (value: PasswordPreset[]) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.passwordPresets);
    });
  },
  get value() {
    return settingsManager.current.passwordPresets;
  },
  addPreset: (preset: PasswordPreset) => {
    settingsManager.update((settings) => {
      settings.passwordPresets = [...settings.passwordPresets, preset];
    });
  },
  deletePreset: (name: string) => {
    settingsManager.update((settings) => {
      settings.passwordPresets = settings.passwordPresets.filter((preset) => preset.name !== name);
    });
  },
  updatePreset: (name: string, updatedPreset: PasswordPreset) => {
    settingsManager.update((settings) => {
      settings.passwordPresets = settings.passwordPresets.map((preset) =>
        preset.name === name ? updatedPreset : preset
      );
    });
  },
  resetPresets: () => {
    settingsManager.update((settings) => {
      settings.passwordPresets = [...defaultPasswordPresets];
    });
  }
};
