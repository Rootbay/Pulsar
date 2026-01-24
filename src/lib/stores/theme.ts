import { derived, writable } from 'svelte/store';
import * as fastDeepEqual from 'fast-deep-equal';
const deepEqual = fastDeepEqual.default;
import { settingsStore } from './settingsStore';
import { appSettings } from './appSettings';

export const theme = derived(appSettings, ($appSettings) => {
  return $appSettings?.appearance?.theme || 'dark';
});

const currentThemeWritable = writable<'light' | 'dark' | 'system'>();

appSettings.subscribe(($appSettings) => {
  if ($appSettings && $appSettings.appearance) {
    currentThemeWritable.set($appSettings.appearance.theme);
  }
});

if (typeof window !== 'undefined') {
  currentThemeWritable.subscribe((value) => {
    const isDark =
      value === 'dark' ||
      (value === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    document.body.classList.toggle('dark', isDark);
  });
}

export const hasUnsavedChanges = derived(
  [appSettings, currentThemeWritable],
  ([$appSettings, $currentThemeWritable]) => {
    return !deepEqual($appSettings.appearance.theme, $currentThemeWritable);
  }
);

function save() {
  currentThemeWritable.subscribe((value) => {
    appSettings.update((settings) => {
      settings.appearance.theme = value;
      return settings;
    });
  })();
}

function reset() {
  appSettings.subscribe((value) => {
    currentThemeWritable.set(value.appearance.theme);
  })();
}

settingsStore.registerModule('theme', {
  save,
  reset,
  hasUnsavedChanges
});

export function setTheme(newTheme: 'light' | 'dark' | 'system') {
  currentThemeWritable.set(newTheme);
}
