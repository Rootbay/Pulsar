import { derived } from 'svelte/store';
import { generalSettings } from '$lib/stores/general';

export type Locale = 'en' | 'sv';

export const currentLocale = derived(generalSettings, ($general): Locale => {
  return $general.appLanguage === 'sv' ? 'sv' : 'en';
});
