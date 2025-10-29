import { derived, writable } from 'svelte/store';
import { appSettings } from './appSettings';
import { defaultVaultSettings, type VaultSettings } from '../config/settings';

function createVaultSettingsStore() {
    const currentVaultId = writable<string | null>(null);
    let activeVaultId: string | null = null;

    currentVaultId.subscribe((id) => {
        activeVaultId = id;
    });

    const store = derived([appSettings, currentVaultId], ([$appSettings, id]) => {
        if (!id) {
            return { ...defaultVaultSettings };
        }

        const existing = $appSettings.vaultSettingsById[id];
        return { ...defaultVaultSettings, ...existing };
    });

    return {
        subscribe: store.subscribe,
        selectVault: (vaultId: string, defaults: Partial<VaultSettings> = {}) => {
            currentVaultId.set(vaultId);
            appSettings.update((settings) => {
                const existing = settings.vaultSettingsById[vaultId];
                settings.vaultSettingsById = {
                    ...settings.vaultSettingsById,
                    [vaultId]: {
                        ...defaultVaultSettings,
                        ...existing,
                        ...defaults,
                    },
                };
                return settings;
            });
        },
        update: (updater: (settings: VaultSettings) => VaultSettings) => {
            if (!activeVaultId) {
                return;
            }

            appSettings.update((settings) => {
                const existing = settings.vaultSettingsById[activeVaultId];
                settings.vaultSettingsById = {
                    ...settings.vaultSettingsById,
                    [activeVaultId]: updater({
                        ...defaultVaultSettings,
                        ...existing,
                    }),
                };
                return settings;
            });
        },
        clear: (vaultId: string) => {
            appSettings.update((settings) => {
                if (!(vaultId in settings.vaultSettingsById)) {
                    return settings;
                }

                const { [vaultId]: _removed, ...rest } = settings.vaultSettingsById;
                settings.vaultSettingsById = rest;
                return settings;
            });

            if (activeVaultId === vaultId) {
                currentVaultId.set(null);
            }
        },
        getCurrentVaultId: () => activeVaultId,
    };
}

export const vaultSettings = createVaultSettingsStore();
