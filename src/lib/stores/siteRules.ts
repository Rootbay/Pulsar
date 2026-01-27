import { appSettings, settingsManager } from './appSettings.svelte';
import { type SiteRule, defaultSiteRules } from '../config/settings';

export const siteRules = {
  subscribe(fn: (value: SiteRule[]) => void) {
    return appSettings.subscribe((settings) => {
      fn(settings.siteRules);
    });
  },
  get value() {
    return settingsManager.current.siteRules;
  },
  addRule: (rule: SiteRule) => {
    settingsManager.update((settings) => {
      settings.siteRules = [...settings.siteRules, rule];
    });
  },
  deleteRule: (url: string) => {
    settingsManager.update((settings) => {
      settings.siteRules = settings.siteRules.filter((rule) => rule.url !== url);
    });
  },
  updateRule: (url: string, updatedRule: SiteRule) => {
    settingsManager.update((settings) => {
      settings.siteRules = settings.siteRules.map((rule) =>
        rule.url === url ? updatedRule : rule
      );
    });
  },
  resetRules: () => {
    settingsManager.update((settings) => {
      settings.siteRules = [...defaultSiteRules];
    });
  }
};
