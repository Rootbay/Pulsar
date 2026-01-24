import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import { type PasswordPreset, defaultPasswordPresets } from '../config/settings';

function createPasswordPresetsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.passwordPresets);

  return {
    subscribe,
    addPreset: (preset: PasswordPreset) => {
      appSettings.update((settings) => {
        settings.passwordPresets = [...settings.passwordPresets, preset];
        return settings;
      });
    },
    deletePreset: (name: string) => {
      appSettings.update((settings) => {
        settings.passwordPresets = settings.passwordPresets.filter(
          (preset) => preset.name !== name
        );
        return settings;
      });
    },
    updatePreset: (name: string, updatedPreset: PasswordPreset) => {
      appSettings.update((settings) => {
        settings.passwordPresets = settings.passwordPresets.map((preset) =>
          preset.name === name ? updatedPreset : preset
        );
        return settings;
      });
    },
    resetPresets: () => {
      appSettings.update((settings) => {
        settings.passwordPresets = [...defaultPasswordPresets];
        return settings;
      });
    }
  };
}

export const passwordPresets = createPasswordPresetsStore();
