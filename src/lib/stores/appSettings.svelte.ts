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
import { defaultKeybinds } from '../config/keybinds';
import type { AllSettings } from '../config/settings';

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

class SettingsStore {
  state = $state<AllSettings>(defaultAllSettings);
  isInitialized = $state(false);
  isSaving = $state(false);
  #saveTimeout: ReturnType<typeof setTimeout> | null = null;
  initPromise: Promise<void>;

  constructor() {
    console.log('[SettingsStore] Constructor called');
    this.initPromise = this.init();
  }

  async init() {
    console.log('[SettingsStore] init called');
    if (this.isInitialized) return;

    try {
      console.log('[SettingsStore] calling backend');
      const storedSettings = await callBackend<string | null>('get_all_settings');
      console.log('[SettingsStore] backend returned', storedSettings);
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
            this.state = this.#deepMerge(
              defaultAllSettings as unknown as Record<string, unknown>,
              loaded as Record<string, unknown>
            );
            callBackend('apply_system_settings').catch(console.error);
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

  #isObject(item: unknown): item is Record<string, unknown> {
    return !!item && typeof item === 'object' && !Array.isArray(item);
  }

  #deepMerge(target: Record<string, unknown>, source: Record<string, unknown>): AllSettings {
    if (!this.#isObject(target) || !this.#isObject(source)) {
      return (source !== undefined ? source : target) as unknown as AllSettings;
    }

    const output: Record<string, unknown> = { ...target };
    Object.keys(source).forEach((key) => {
      if (this.#isObject(source[key])) {
        if (!(key in target)) {
          output[key] = source[key];
        } else {
          output[key] = this.#deepMerge(
            target[key] as Record<string, unknown>,
            source[key] as Record<string, unknown>
          );
        }
      } else {
        output[key] = source[key];
      }
    });
    return output as unknown as AllSettings;
  }

  async saveNow() {
    if (this.#saveTimeout) {
      clearTimeout(this.#saveTimeout);
      this.#saveTimeout = null;
    }

    try {
      const snapshot = $state.snapshot(this.state);
      await callBackend('set_all_settings', { settings: JSON.stringify(snapshot) });
      await callBackend('apply_system_settings');
    } catch (error) {
      console.error('Failed to save settings:', error);
    } finally {
      this.isSaving = false;
    }
  }

  save() {
    if (this.#saveTimeout) clearTimeout(this.#saveTimeout);

    this.#saveTimeout = setTimeout(() => {
      this.saveNow();
    }, 500);
  }

  update<K extends keyof AllSettings>(key: K, value: AllSettings[K]) {
    this.state[key] = value;
    this.save();
  }

  setState(newState: AllSettings) {
    this.state = newState;
    this.saveNow();
  }
}

export const settings = new SettingsStore();
export const initAppSettings = () => settings.initPromise;
