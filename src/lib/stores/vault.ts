import { derived } from 'svelte/store';
import { appSettings } from './appSettings';
import type { VaultSettings } from '../config/settings';

function createVaultSettingsStore() {
    const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.vault);

    return {
        subscribe,
        set: (value: VaultSettings) => {
            appSettings.update((settings) => {
                settings.vault = value;
                return settings;
            });
        },
        loadSettings: (newSettings: VaultSettings) => {
            appSettings.update((settings) => {
                settings.vault = newSettings;
                return settings;
            });
        },
        update: (callback: (settings: VaultSettings) => VaultSettings) => {
            appSettings.update((settings) => {
                settings.vault = callback(settings.vault);
                return settings;
            });
        }
    };
}

export const vaultSettings = createVaultSettingsStore();