import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import { defaultKeybinds } from '../config/keybinds';

function createKeybindsStore() {
  const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.keybinds);

  return {
    subscribe,
    updateKeybind: (name: string, newKey: string) => {
      appSettings.update((settings) => {
        const index = settings.keybinds.findIndex((kb) => kb.name === name);
        if (index !== -1) {
          settings.keybinds[index].key = newKey;
        }
        return settings;
      });
    },
    resetKeybinds: () => {
      appSettings.update((settings) => {
        settings.keybinds = [...defaultKeybinds];
        return settings;
      });
    }
  };
}

export const keybinds = createKeybindsStore();
