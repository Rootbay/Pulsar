import { derived } from 'svelte/store';
import { generalSettings } from '$lib/stores/general';
import en from './i18n/en.json';
import sv from './i18n/sv.json';

export type Locale = 'en' | 'sv';
type Messages = typeof en;
export type I18nKey = keyof Messages;

const dictionaries: Record<Locale, Messages> = { en, sv };

export const currentLocale = derived(generalSettings, ($general): Locale => {
  return $general.appLanguage === 'sv' ? 'sv' : 'en';
});

export function t(
  locale: Locale,
  key: I18nKey,
  vars: Record<string, string | number> = {}
): string {
  const dictionary = dictionaries[locale] ?? dictionaries.en;
  const template = dictionary[key] ?? dictionaries.en[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (match, token) => {
    if (Object.prototype.hasOwnProperty.call(vars, token)) {
      return String(vars[token]);
    }
    return match;
  });
}
