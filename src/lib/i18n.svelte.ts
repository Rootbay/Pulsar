import { settings } from '$lib/stores/appSettings.svelte';
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
import tr from './i18n/tr.json';
import nl from './i18n/nl.json';
import pl from './i18n/pl.json';
import id from './i18n/id.json';
import th from './i18n/th.json';
import vi from './i18n/vi.json';
import el from './i18n/el.json';

export type Locale =
  | 'en'
  | 'sv'
  | 'es'
  | 'fr'
  | 'de'
  | 'pt-BR'
  | 'zh'
  | 'ru'
  | 'ja'
  | 'hi'
  | 'ko'
  | 'ar'
  | 'it'
  | 'tr'
  | 'nl'
  | 'pl'
  | 'id'
  | 'th'
  | 'vi'
  | 'el';

type Messages = typeof en;
export type I18nKey = keyof Messages;

const dictionaries: Record<Locale, Partial<Messages>> = {
  en,
  sv,
  es,
  fr,
  de,
  'pt-BR': ptBr,
  zh,
  ru,
  ja,
  hi,
  ko,
  ar,
  it,
  tr,
  nl,
  pl,
  id,
  th,
  vi,
  el
};

const supportedLocales = new Set<Locale>(Object.keys(dictionaries) as Locale[]);
const supportedLocaleList = Object.keys(dictionaries) as Locale[];
const supportedLocaleByLower = new Map(
  supportedLocaleList.map((locale) => [locale.toLowerCase(), locale])
);

function normalizeLocale(raw: string | null | undefined): Locale | null {
  if (!raw) return null;
  const normalized = raw.replace('_', '-').toLowerCase();
  const direct = supportedLocaleByLower.get(normalized);
  if (direct) return direct;

  const [language, region] = normalized.split('-');
  if (language === 'pt') return 'pt-BR';
  if (language === 'zh') return 'zh';

  const base = supportedLocaleByLower.get(language);
  if (base) return base;

  if (language === 'en' && region) return 'en';
  return null;
}

function detectSystemLocale(): Locale {
  const candidates: Array<string | undefined> = [];
  if (typeof navigator !== 'undefined') {
    candidates.push(...(navigator.languages ?? []));
    candidates.push(navigator.language);
  }
  if (typeof Intl !== 'undefined') {
    candidates.push(Intl.DateTimeFormat().resolvedOptions().locale);
  }

  for (const candidate of candidates) {
    const match = normalizeLocale(candidate);
    if (match) return match;
  }

  return 'en';
}

export const i18n = {
  get locale(): Locale {
    const preferred = settings.state.general.appLanguage;
    if (preferred === 'system') {
      return detectSystemLocale();
    }
    const normalized = normalizeLocale(preferred);
    return normalized && supportedLocales.has(normalized) ? normalized : 'en';
  }
};

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

export const currentLocale = {
  get value() {
    return i18n.locale;
  },
  subscribe(fn: (v: Locale) => void) {
    fn(i18n.locale);
    return $effect.root(() => {
      $effect(() => fn(i18n.locale));
    });
  }
};
