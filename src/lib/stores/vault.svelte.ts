import { settingsManager } from './appSettings.svelte';
import { defaultVaultSettings, type VaultSettings } from '../config/settings';

let activeVaultId = $state<string | null>(null);

export const vaultSettings = {
  subscribe(fn: (value: VaultSettings) => void) {
    return $effect.root(() => {
      $effect(() => {
        const id = activeVaultId;
        const allSettings = settingsManager.current;
        if (!id) {
          fn({ ...defaultVaultSettings });
          return;
        }
        const existing = allSettings.vaultSettingsById[id];
        fn({ ...defaultVaultSettings, ...existing });
      });
    });
  },
  get value() {
    if (!activeVaultId) return { ...defaultVaultSettings };
    const existing = settingsManager.current.vaultSettingsById[activeVaultId];
    return { ...defaultVaultSettings, ...existing };
  },
  selectVault(vaultId: string, defaults: Partial<VaultSettings> = {}) {
    activeVaultId = vaultId;
    settingsManager.update((settings) => {
      const existing = settings.vaultSettingsById[vaultId];
      settings.vaultSettingsById = {
        ...settings.vaultSettingsById,
        [vaultId]: {
          ...defaultVaultSettings,
          ...existing,
          ...defaults
        }
      };
    });
  },
  update(updater: (settings: VaultSettings) => VaultSettings) {
    if (!activeVaultId) return;
    const id = activeVaultId; // Capture for closure
    settingsManager.update((settings) => {
      const existing = settings.vaultSettingsById[id];
      settings.vaultSettingsById = {
        ...settings.vaultSettingsById,
        [id]: updater({
          ...defaultVaultSettings,
          ...existing
        })
      };
    });
  },
  clear(vaultId: string) {
    settingsManager.update((settings) => {
      if (!(vaultId in settings.vaultSettingsById)) return;
      const newMap = { ...settings.vaultSettingsById };
      delete newMap[vaultId];
      settings.vaultSettingsById = newMap;
    });
    if (activeVaultId === vaultId) {
      activeVaultId = null;
    }
  },
  getCurrentVaultId: () => activeVaultId
};
