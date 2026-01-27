import { browser } from '$app/environment';
import { settings } from './appSettings.svelte';
import { callBackend } from '../utils/backend';

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

  constructor() {
    $effect.root(() => {
      $effect(() => {
        if (this.isLocked) {
          this.resetUI();
        }
      });

      if (browser) {
        this.#setupActivityListeners();
        
        $effect(() => {
          if (!this.isLocked && this.isDatabaseLoaded) {
            const interval = setInterval(() => this.#checkInactivity(), 10000);
            return () => clearInterval(interval);
          }
        });
      }
    });
  }

  #setupActivityListeners() {
    const handleActivity = () => {
      this.lastActivity = Date.now();
    };

    window.addEventListener('mousemove', handleActivity);
    window.addEventListener('keydown', handleActivity);
    window.addEventListener('mousedown', handleActivity);
    window.addEventListener('touchstart', handleActivity);
    window.addEventListener('scroll', handleActivity);
  }

  #checkInactivity() {
    const autoLockSetting = settings.state.security.autoLockInactivity;
    if (!autoLockSetting || autoLockSetting === 'never') return;

    const timeoutMs = this.#parseInactivityString(autoLockSetting);
    if (timeoutMs === 0) return;

    if (Date.now() - this.lastActivity > timeoutMs) {
      console.log('Auto-locking due to inactivity');
      this.lock();
    }
  }

  #parseInactivityString(s: string): number {
    const match = s.match(/(\d+)\s*(minute|hour|second)s?/i);
    if (!match) return 0;

    const value = parseInt(match[1]);
    const unit = match[2].toLowerCase();

    switch (unit) {
      case 'second': return value * 1000;
      case 'minute': return value * 60 * 1000;
      case 'hour': return value * 60 * 60 * 1000;
      default: return 0;
    }
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
  }
}

export const appState = new AppState();
