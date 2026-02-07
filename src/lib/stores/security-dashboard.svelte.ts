import { SvelteSet } from 'svelte/reactivity';
import type { PasswordItem, PasswordItemOverview } from '$lib/types/password';
import { SecurityService, type PasswordHealth } from '../utils/security';
import { appState } from './appState.svelte';
import { callBackend } from '../utils/backend';

export interface SecurityReport {
  reusedPasswords: { passwordHash: string; itemIds: number[]; count: number }[];
  weakPasswords: number[];
  breachedPasswords: number[];
  uniquePasswordsCount: number;
  totalPasswordsCount: number;
  overallHealthScore: number;
}

export interface SecurityDashboardState {
  items: Record<number, PasswordHealth>;
  lastReport: SecurityReport | null;
  problematicItems: PasswordItemOverview[];
}

const initialState: SecurityDashboardState = {
  items: {},
  lastReport: null,
  problematicItems: []
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

  get lastReport() {
    return this.#state.lastReport;
  }

  get problematicItems() {
    return this.#state.problematicItems;
  }

  reset() {
    this.#state = {
      items: {},
      lastReport: null,
      problematicItems: []
    };
  }

  async runAudit() {
    try {
      const report = await callBackend<SecurityReport>('get_security_report');
      this.#state.lastReport = report;

      const allProblematicIds = new SvelteSet<number>();
      report.weakPasswords.forEach((id) => allProblematicIds.add(id));
      report.breachedPasswords.forEach((id) => allProblematicIds.add(id));
      report.reusedPasswords.forEach((group) =>
        group.itemIds.forEach((id) => allProblematicIds.add(id))
      );

      if (allProblematicIds.size > 0) {
        this.#state.problematicItems = await callBackend<PasswordItemOverview[]>(
          'get_password_overviews_by_ids',
          { ids: Array.from(allProblematicIds) }
        );
      } else {
        this.#state.problematicItems = [];
      }

      return report;
    } catch (error) {
      console.error('Audit failed:', error);
      throw error;
    }
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
    
    // Guard against unnecessary updates that could cause infinite loops in Svelte 5 effects
    if (
      currentHealth.score === result.score &&
      currentHealth.crackTimeDisplay === result.crackTimesDisplay.offlineSlowHashing1e4PerSecond &&
      currentHealth.warning === (result.feedback.warning || '') &&
      JSON.stringify(currentHealth.suggestions) === JSON.stringify(result.feedback.suggestions)
    ) {
      return;
    }

    this.#state.items[item.id] = {
      ...currentHealth,
      score: result.score,
      crackTimeDisplay: result.crackTimesDisplay.offlineSlowHashing1e4PerSecond,
      suggestions: result.feedback.suggestions,
      warning: result.feedback.warning || ''
    };
  }

  async checkBreach(item: PasswordItem) {
    if (!item.password || !item.id) return;

    const count = await SecurityService.checkBreach(item.password);

    const current = this.#state.items[item.id];
    if (current && (current.isBreached !== (count > 0) || current.breachCount !== count)) {
      this.#state.items[item.id] = {
        ...current,
        isBreached: count > 0,
        breachCount: count
      };
    }
  }
}

export const securityDashboard = new SecurityDashboardStore();
