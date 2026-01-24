import { callBackend } from '../utils/backend';
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
  type VaultSettingsMap,
  defaultVaultSettingsMap,
  type PasswordPreset,
  defaultPasswordPresets,
  type SiteRule,
  defaultSiteRules
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
          let loadedSettings: any;
          if (storedSettings.startsWith('"') && storedSettings.endsWith('"')) {
            loadedSettings = JSON.parse(JSON.parse(storedSettings));
          } else {
            loadedSettings = JSON.parse(storedSettings);
          }

          if (typeof loadedSettings === 'object' && loadedSettings !== null) {
            this.#state = {
              ...defaultAllSettings,
              ...loadedSettings,
              advanced: { ...defaultAllSettings.advanced, ...(loadedSettings.advanced || {}) },
              appearance: {
                ...defaultAllSettings.appearance,
                ...(loadedSettings.appearance || {})
              },
              autofill: { ...defaultAllSettings.autofill, ...(loadedSettings.autofill || {}) },
              backup: { ...defaultAllSettings.backup, ...(loadedSettings.backup || {}) },
              clipboard: { ...defaultAllSettings.clipboard, ...(loadedSettings.clipboard || {}) },
              general: { ...defaultAllSettings.general, ...(loadedSettings.general || {}) },
              generator: { ...defaultAllSettings.generator, ...(loadedSettings.generator || {}) },
              security: { ...defaultAllSettings.security, ...(loadedSettings.security || {}) },
              vaultSettingsById: {
                ...defaultAllSettings.vaultSettingsById,
                ...(loadedSettings.vaultSettingsById || loadedSettings.vault || {})
              }
            };
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
          const currentState = this.#state;
          if (this.#isInitialized) {
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
  subscribe(fn: (value: AllSettings) => void) {
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
