import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import {
    type AdvancedSettings,
    defaultAdvancedSettings,
    type AppearanceSettings,
    defaultAppearanceSettings,
    type AutofillSettings,
    defaultAutofillSettings,
    type BackupSettings,
    defaultBackupSettings,
    type ClipboardSettings,
    defaultClipboardSettings,
    type GeneralSettings,
    defaultGeneralSettings,
    type GeneratorSettings,
    defaultGeneratorSettings,
    type SecuritySettings,
    defaultSecuritySettings,
    type VaultSettings,
    defaultVaultSettings,
    type PasswordPreset,
    defaultPasswordPresets,
    type SiteRule,
    defaultSiteRules,
} from '../config/settings';
import { type Keybind, defaultKeybinds } from '../config/keybinds';

export interface AllSettings {
    advanced: AdvancedSettings;
    appearance: AppearanceSettings;
    autofill: AutofillSettings;
    backup: BackupSettings;
    clipboard: ClipboardSettings;
    general: GeneralSettings;
    generator: GeneratorSettings;
    keybinds: Keybind[];
    passwordPresets: PasswordPreset[];
    recentDatabases: string[];
    siteRules: SiteRule[];
    security: SecuritySettings;
    vault: VaultSettings;
}

const defaultAllSettings: AllSettings = {
    advanced: defaultAdvancedSettings,
    appearance: defaultAppearanceSettings,
    autofill: defaultAutofillSettings,
    backup: defaultBackupSettings,
    clipboard: defaultClipboardSettings,
    general: defaultGeneralSettings,
    generator: defaultGeneratorSettings,
    keybinds: defaultKeybinds,
    passwordPresets: defaultPasswordPresets,
    recentDatabases: [],
    siteRules: defaultSiteRules,
    security: defaultSecuritySettings,
    vault: defaultVaultSettings,
};

export const appSettings = writable<AllSettings>(defaultAllSettings);

let isInitialized = false;

export async function initAppSettings() {
    if (isInitialized) return;

    try {
        const storedSettings = await invoke<string | null>('get_all_settings');
        if (storedSettings) {
            try {
                const loadedSettings = JSON.parse(storedSettings);

                if (typeof loadedSettings === 'object' && loadedSettings !== null) {
                    const mergedSettings = {
                        ...defaultAllSettings,
                        ...loadedSettings,
                        advanced: { ...defaultAllSettings.advanced, ...(loadedSettings.advanced || {}) },
                        appearance: { ...defaultAllSettings.appearance, ...(loadedSettings.appearance || {}) },
                        autofill: { ...defaultAllSettings.autofill, ...(loadedSettings.autofill || {}) },
                        backup: { ...defaultAllSettings.backup, ...(loadedSettings.backup || {}) },
                        clipboard: { ...defaultAllSettings.clipboard, ...(loadedSettings.clipboard || {}) },
                        general: { ...defaultAllSettings.general, ...(loadedSettings.general || {}) },
                        generator: { ...defaultAllSettings.generator, ...(loadedSettings.generator || {}) },
                        security: { ...defaultAllSettings.security, ...(loadedSettings.security || {}) },
                        vault: { ...defaultAllSettings.vault, ...(loadedSettings.vault || {}) },
                        keybinds: loadedSettings.keybinds || defaultAllSettings.keybinds,
                        passwordPresets: loadedSettings.passwordPresets || defaultAllSettings.passwordPresets,
                        recentDatabases: loadedSettings.recentDatabases || defaultAllSettings.recentDatabases,
                        siteRules: loadedSettings.siteRules || defaultAllSettings.siteRules,
                    };
                    appSettings.set(mergedSettings);
                } else {
                    // loaded settings is not an object (e.g. "null" string was stored)
                    appSettings.set(defaultAllSettings);
                }
            } catch (e) {
                // parsing failed
                console.error("Failed to parse stored settings:", e);
                appSettings.set(defaultAllSettings);
            }
        } else {
            // No settings found, save defaults
            await invoke('set_all_settings', { settings: JSON.stringify(defaultAllSettings) });
        }
    } catch (error) {
        console.error('Failed to load settings:', error);
        // Fallback to defaults if loading fails
        appSettings.set(defaultAllSettings);
    } finally {
        isInitialized = true;
    }
}

let saveTimeout: ReturnType<typeof setTimeout>;
appSettings.subscribe(async (currentSettings) => {
    if (!isInitialized) return; // Don't save until initialized

    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
        try {
            await invoke('set_all_settings', { settings: JSON.stringify(currentSettings) });
        } catch (error) {
            console.error('Failed to save settings:', error);
        }
    }, 500); // Debounce saving for 500ms
});
