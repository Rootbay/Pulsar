import { writable } from 'svelte/store';
import { theme } from './stores/theme';
import { settingsStore } from './stores/settingsStore';

export type FilterCategory = 'all' | 'recent';

export const selectedTag = writable<string | null>(null);
export const currentView = writable<string>('passwords');
export const filterCategory = writable<FilterCategory>('all');
export const isLocked = writable<boolean>(true);
export const isDatabaseLoaded = writable<boolean>(false);
export const needsPasswordSetup = writable<boolean>(false);
export const showSettingsPopup = writable<boolean>(false);
export const totpVerified = writable<boolean>(false);

export { theme, settingsStore };
