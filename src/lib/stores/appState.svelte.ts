import { callBackend } from '../utils/backend';
import { loginTotpStore } from './totp.svelte';
import { profileSettings, defaultProfileSettings } from './profile.svelte';

export type FilterCategory = 'all' | 'recent' | 'favorites';

class AppState {
  selectedTag = $state<string | null>(null);
  currentView = $state<string>('passwords');
  filterCategory = $state<FilterCategory>('all');
  isLocked = $state<boolean>(true);
  isDatabaseLoaded = $state<boolean>(false);
  needsPasswordSetup = $state<boolean>(false);
  showSettingsPopup = $state<boolean>(false);
  totpVerified = $state<boolean>(false);
  totpRequired = $state<boolean>(false);
  lastActivity = $state<number>(Date.now());
  requestedItemId = $state<number | null>(null);

  constructor() {
    $effect.root(() => {
      $effect(() => {
        if (this.isLocked) {
          this.resetUI();
        }
      });
    });
  }

  async lock() {
    try {
      await callBackend('lock');
      this.isLocked = true;
      this.totpVerified = false;
    } catch (error) {
      console.error('Failed to lock vault:', error);
    }
  }

  get isReady() {
    return this.isDatabaseLoaded && !this.isLocked && (!this.totpRequired || this.totpVerified);
  }

  resetUI() {
    this.selectedTag = null;
    this.currentView = 'passwords';
    this.filterCategory = 'all';
    loginTotpStore.reset();
    profileSettings.state = { ...defaultProfileSettings };
  }
}

export const appState = new AppState();
