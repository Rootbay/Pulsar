import { derived } from 'svelte/store';
import { generalSettings } from '$lib/stores/general';
import en from './i18n/en.json';
import sv from './i18n/sv.json';
import es from './i18n/es.json';
import fr from './i18n/fr.json';
import de from './i18n/de.json';
import ptBr from './i18n/pt-BR.json';
import zh from './i18n/zh.json';
import ru from './i18n/ru.json';
import ja from './i18n/ja.json';
import hi from './i18n/hi.json';
import ko from './i18n/ko.json';
import ar from './i18n/ar.json';
import it from './i18n/it.json';

export type Locale = 'en' | 'sv' | 'es' | 'fr' | 'de' | 'pt-BR' | 'zh' | 'ru' | 'ja' | 'hi' | 'ko' | 'ar' | 'it';
type Messages = typeof en;
export type I18nKey = keyof Messages;

const dictionaries: Record<Locale, Messages> = { en, sv, es, fr, de, 'pt-BR': ptBr, zh, ru, ja, hi, ko, ar, it };
const supportedLocales = new Set<Locale>(Object.keys(dictionaries) as Locale[]);

export const currentLocale = derived(generalSettings, ($general): Locale => {
  const preferred = $general.appLanguage as Locale;
  return supportedLocales.has(preferred) ? preferred : 'en';
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
