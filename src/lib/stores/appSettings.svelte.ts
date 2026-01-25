import { callBackend } from '../utils/backend';
import {
  defaultAdvancedSettings,
  defaultAppearanceSettings,
  defaultAutofillSettings,
  defaultBackupSettings,
  defaultClipboardSettings,
  defaultGeneralSettings,
  defaultGeneratorSettings,
  defaultSecuritySettings,
  defaultVaultSettingsMap,
  defaultPasswordPresets,
  defaultSiteRules
} from '../config/settings';
import type {
  AdvancedSettings,
  AppearanceSettings,
  AutofillSettings,
  BackupSettings,
  ClipboardSettings,
  GeneralSettings,
  GeneratorSettings,
  SecuritySettings,
  VaultSettingsMap,
  PasswordPreset,
  SiteRule
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
  vaultSettingsById: VaultSettingsMap;
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
  vaultSettingsById: defaultVaultSettingsMap
};

class SettingsManager {
  #state = $state<AllSettings>(defaultAllSettings);
  #isInitialized = false;
  #saveTimeout: ReturnType<typeof setTimeout> | null = null;

  get current() {
    return this.#state;
  }

  async init() {
    if (this.#isInitialized) return;

    try {
      const storedSettings = await callBackend<string | null>('get_all_settings');
      if (storedSettings) {
        try {
          let loadedSettings: unknown;
          if (storedSettings.startsWith('"') && storedSettings.endsWith('"')) {
            loadedSettings = JSON.parse(JSON.parse(storedSettings));
          } else {
            loadedSettings = JSON.parse(storedSettings);
          }

          if (typeof loadedSettings === 'object' && loadedSettings !== null) {
            const settings = loadedSettings as Record<string, Record<string, unknown>>;
            this.#state = {
              ...defaultAllSettings,
              ...(loadedSettings as Record<string, unknown>),
              advanced: {
                ...defaultAllSettings.advanced,
                ...(settings.advanced || {})
              },
              appearance: {
                ...defaultAllSettings.appearance,
                ...(settings.appearance || {})
              },
              autofill: { ...defaultAllSettings.autofill, ...(settings.autofill || {}) },
              backup: { ...defaultAllSettings.backup, ...(settings.backup || {}) },
              clipboard: { ...defaultAllSettings.clipboard, ...(settings.clipboard || {}) },
              general: { ...defaultAllSettings.general, ...(settings.general || {}) },
              generator: { ...defaultAllSettings.generator, ...(settings.generator || {}) },
              security: { ...defaultAllSettings.security, ...(settings.security || {}) },
              vaultSettingsById: {
                ...defaultAllSettings.vaultSettingsById,
                ...(settings.vaultSettingsById || (settings.vault as Record<string, unknown>) || {})
              }
            } as AllSettings;
          }
        } catch (e) {
          console.error('Failed to parse stored settings:', e);
        }
      } else {
        await this.save();
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      this.#isInitialized = true;

      $effect.root(() => {
        $effect(() => {
          if (this.#isInitialized) {
            void this.#state; // Subscribe
            this.scheduleSave();
          }
        });
      });
    }
  }

  scheduleSave() {
    if (this.#saveTimeout) clearTimeout(this.#saveTimeout);
    this.#saveTimeout = setTimeout(() => this.save(), 500);
  }

  async save() {
    try {
      await callBackend('set_all_settings', { settings: JSON.stringify(this.#state) });
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }

  update(updater: (settings: AllSettings) => void) {
    updater(this.#state);
    this.scheduleSave();
  }
}

export const settingsManager = new SettingsManager();

export const appSettings = {
  get(): AllSettings {
    return settingsManager.current;
  },
  subscribe(fn: (value: AllSettings) => void) {
    fn(settingsManager.current);
    return $effect.root(() => {
      $effect(() => fn(settingsManager.current));
    });
  },
  update(updater: (settings: AllSettings) => AllSettings) {
    settingsManager.update((s) => {
      const next = updater(s);
      Object.assign(s, next);
    });
  },
  set(value: AllSettings) {
    settingsManager.update((s) => Object.assign(s, value));
  }
};

export const initAppSettings = () => settingsManager.init();
