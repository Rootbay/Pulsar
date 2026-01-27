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
import type { AllSettings } from '../config/settings';

const defaultAllSettings: AllSettings = {
  advanced: defaultAdvancedSettings,
  appearance: defaultAppearanceSettings,
  autofill: defaultAutofillSettings,
  backup: defaultBackupSettings,
  clipboard: defaultClipboardSettings,
  general: defaultGeneralSettings,
  generator: defaultGeneratorSettings,
  keybinds: [],
  passwordPresets: defaultPasswordPresets,
  recentDatabases: [],
  siteRules: defaultSiteRules,
  security: defaultSecuritySettings,
  vaultSettingsById: defaultVaultSettingsMap
};

class SettingsStore {
  state = $state<AllSettings>(defaultAllSettings);
  isInitialized = $state(false);
  isSaving = $state(false);
  #saveTimeout: ReturnType<typeof setTimeout> | null = null;
  initPromise: Promise<void>;

  constructor() {
    this.initPromise = this.init();
  }

  async init() {
    if (this.isInitialized) return;

    try {
      const storedSettings = await callBackend<string | null>('get_all_settings');
      if (storedSettings) {
        try {
          let loaded: unknown;
          try {
            const firstPass = JSON.parse(storedSettings);
            if (typeof firstPass === 'string') {
              loaded = JSON.parse(firstPass);
            } else {
              loaded = firstPass;
            }
          } catch {
             console.warn('Settings corrupted, using defaults');
          }

          if (loaded && typeof loaded === 'object') {
            this.state = this.#mergeDefaults(loaded as Partial<AllSettings>);
          }
        } catch (e) {
          console.error('Failed to parse stored settings:', e);
        }
      } else {
        this.save();
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      this.isInitialized = true;
    }
  }

  #mergeDefaults(loaded: Partial<AllSettings>): AllSettings {
     const merged = { ...defaultAllSettings };
     
     for (const key of Object.keys(defaultAllSettings) as (keyof AllSettings)[]) {
         if (loaded[key]) {
             if (typeof defaultAllSettings[key] === 'object' && !Array.isArray(defaultAllSettings[key]) && loaded[key]) {
                 // @ts-ignore
                 merged[key] = { ...defaultAllSettings[key], ...loaded[key] };
             } else {
                 // @ts-ignore
                 merged[key] = loaded[key];
             }
         }
     }
     return merged;
  }

  save() {
    if (this.#saveTimeout) clearTimeout(this.#saveTimeout);
    
    this.#saveTimeout = setTimeout(async () => {
        this.isSaving = true;
        try {
          await callBackend('set_all_settings', { settings: JSON.stringify(this.state) });
        } catch (error) {
          console.error('Failed to save settings:', error);
        } finally {
          this.isSaving = false;
        }
    }, 500);
  }

  update<K extends keyof AllSettings>(key: K, value: AllSettings[K]) {
      this.state[key] = value;
      this.save();
  }
  
  setState(newState: AllSettings) {
      this.state = newState;
      this.save();
  }
}

export const settings = new SettingsStore();
export const initAppSettings = () => settings.initPromise;

