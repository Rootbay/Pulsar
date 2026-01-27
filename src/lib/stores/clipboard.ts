import { appSettings, settingsManager } from './appSettings.svelte';
import type { ClipboardSettings } from '../config/settings';

export const clipboardSettings = {
  subscribe(fn: (value: ClipboardSettings) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.clipboard);
    });
  },
  set(value: ClipboardSettings) {
    settingsManager.update((s) => {
      s.clipboard = value;
    });
  },
  update(updater: (s: ClipboardSettings) => ClipboardSettings) {
    settingsManager.update((s) => {
      s.clipboard = updater(s.clipboard);
    });
  },
  get value() {
    return settingsManager.current.clipboard;
  },
  set value(v: ClipboardSettings) {
    settingsManager.update((s) => {
      s.clipboard = v;
    });
  }
};
