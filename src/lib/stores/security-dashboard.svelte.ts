import type { PasswordItem } from '$lib/types/password';
import { SecurityService, type PasswordHealth } from '../utils/security';
import { appState } from './appState.svelte';

interface SecurityDashboardState {
  items: Record<number, PasswordHealth>;
}

const initialState: SecurityDashboardState = {
  items: {}
};

class SecurityDashboardStore {
  #state = $state<SecurityDashboardState>(initialState);

  constructor() {
    $effect.root(() => {
      $effect(() => {
        if (appState.isLocked) {
          this.reset();
        }
      });
    });
  }

  get items() {
    return this.#state.items;
  }

  reset() {
    this.#state = { items: {} };
  }

  assessStrength(item: PasswordItem) {
    if (!item.password) return;

    const userInputs = [
      item.username || '',
      item.title || '',
      item.url || '',
      'pulsar',
      ...(item.tags?.split(',') || [])
    ];

    const result = SecurityService.checkStrength(item.password, userInputs);

    const currentHealth = this.#state.items[item.id] || { isBreached: null, breachCount: 0 };
    this.#state.items[item.id] = {
      ...currentHealth,
      score: result.score,
      crackTimeDisplay: result.crackTimesDisplay.offlineSlowHashing1e4PerSecond,
      suggestions: result.feedback.suggestions,
      warning: result.feedback.warning || ''
    };
  }

  async checkBreach(item: PasswordItem) {
    if (!item.password) return;

    const count = await SecurityService.checkBreach(item.password);

    const current = this.#state.items[item.id];
    if (current) {
      this.#state.items[item.id] = {
        ...current,
        isBreached: count > 0,
        breachCount: count
      };
    }
  }
}

const store = new SecurityDashboardStore();

export const securityDashboard = {
  subscribe(fn: (value: SecurityDashboardState) => void) {
    return $effect.root(() => {
      $effect(() => {
        fn({ items: store.items });
      });
    });
  },
  reset: () => store.reset(),
  assessStrength: (item: PasswordItem) => store.assessStrength(item),
  checkBreach: (item: PasswordItem) => store.checkBreach(item)
};