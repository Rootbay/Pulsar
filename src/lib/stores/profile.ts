import { secureStorage } from '$lib/utils/secureStorage';

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
  phone: '+39 555 0100',
};

export const profileSettings = secureStorage.createStore<ProfileSettings>(
  'pulsar.profile',
  defaultProfileSettings
);

