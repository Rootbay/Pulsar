import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import type { ClipboardSettings } from '../config/settings';

function createClipboardSettingsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.clipboard);

  return {
    subscribe,
    set: (value: ClipboardSettings) => {
      appSettings.update((settings) => {
        settings.clipboard = value;
        return settings;
      });
    },
    update: (callback: (settings: ClipboardSettings) => ClipboardSettings) => {
      appSettings.update((settings) => {
        settings.clipboard = callback(settings.clipboard);
        return settings;
      });
    }
  };
}

export const clipboardSettings = createClipboardSettingsStore();
