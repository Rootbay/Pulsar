import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import { type SiteRule, defaultSiteRules } from '../config/settings';

function createSiteRulesStore() {
    const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.siteRules);

    return {
        subscribe,
        addRule: (rule: SiteRule) => {
            appSettings.update((settings) => {
                settings.siteRules = [...settings.siteRules, rule];
                return settings;
            });
        },
        deleteRule: (url: string) => {
            appSettings.update((settings) => {
                settings.siteRules = settings.siteRules.filter(rule => rule.url !== url);
                return settings;
            });
        },
        updateRule: (url: string, updatedRule: SiteRule) => {
            appSettings.update((settings) => {
                settings.siteRules = settings.siteRules.map(rule =>
                    (rule.url === url ? updatedRule : rule)
                );
                return settings;
            });
        },
        resetRules: () => {
            appSettings.update((settings) => {
                settings.siteRules = [...defaultSiteRules];
                return settings;
            });
        }
    };
}

export const siteRules = createSiteRulesStore();
