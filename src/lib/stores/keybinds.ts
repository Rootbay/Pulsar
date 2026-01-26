import { appSettings, settingsManager } from './appSettings.svelte';
import { defaultKeybinds, type Keybind } from '../config/keybinds';

export const keybinds = {
  subscribe(fn: (value: Keybind[]) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.keybinds);
    });
  },
  get value() {
    return settingsManager.current.keybinds;
  },
  updateKeybind: (name: string, newKey: string) => {
    settingsManager.update((settings) => {
      const index = settings.keybinds.findIndex((kb) => kb.name === name);
      if (index !== -1) {
        settings.keybinds[index].key = newKey;
      }
    });
  },
  resetKeybinds: () => {
    settingsManager.update((settings) => {
      settings.keybinds = [...defaultKeybinds];
    });
  }
};

