import { callBackend } from '../utils/backend';
import { appState } from './appState.svelte';

export interface ProfileSettings {
  name: string;
  role: string;
  country: string;
  timezone: string;
  about: string;
  phone: string;
}

export const defaultProfileSettings: ProfileSettings = {
  name: 'David',
  role: 'Vault owner',
  country: 'Italy',
  timezone: 'Europe/Rome',
  about: "Hey, I'm a dev",
  phone: '+39 555 0100'
};

class ProfileStore {
  state = $state<ProfileSettings>(defaultProfileSettings);

  async load() {
    try {
      const data = await callBackend<string | null>('get_profile_settings');
      if (data) {
        this.state = JSON.parse(data);
      }
    } catch (e) {
      console.error('Failed to load profile settings:', e);
    }
  }

  async save() {
    try {
      await callBackend('save_profile_settings', { settingsJson: JSON.stringify(this.state) });
    } catch (e) {
      console.error('Failed to save profile settings:', e);
    }
  }
}

export const profileSettings = new ProfileStore();
