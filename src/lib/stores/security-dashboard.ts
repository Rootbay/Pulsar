import { writable } from 'svelte/store';
import type { PasswordItem } from '$lib/types/password';
import { SecurityService, type PasswordHealth } from '../utils/security';

interface SecurityDashboardState {
    analyzedCount: number;
    breachedCount: number;
    weakCount: number; // Score < 3
    reusedCount: number;
    items: Record<number, PasswordHealth>;
    isScanning: boolean;
}

const initialState: SecurityDashboardState = {
    analyzedCount: 0,
    breachedCount: 0,
    weakCount: 0,
    reusedCount: 0,
    items: {},
    isScanning: false
};

function createSecurityStore() {
    const { subscribe, update, set } = writable<SecurityDashboardState>(initialState);

    return {
        subscribe,
        reset: () => set(initialState),
        
        /**
         * Scans a single item for strength (synchronous/fast)
         */
        assessStrength: (item: PasswordItem) => {
            if (!item.password) return;

            const userInputs = [
                item.username || '',
                item.title || '',
                item.url || '',
                'pulsar', // App name as common input
                ...(item.tags?.split(',') || [])
            ];

            const result = SecurityService.checkStrength(item.password, userInputs);

            update(state => {
                const currentHealth = state.items[item.id] || { isBreached: null, breachCount: 0 };
                state.items[item.id] = {
                    ...currentHealth,
                    score: result.score,
                    crackTimeDisplay: result.crackTimesDisplay.offlineSlowHashing1e4PerSecond,
                    suggestions: result.feedback.suggestions,
                    warning: result.feedback.warning
                };
                return state;
            });
        },

        /**
         * Checks if an item is breached (Async - call sparingly or on demand)
         */
        checkBreach: async (item: PasswordItem) => {
            if (!item.password) return;
            
            // Set loading state if needed in UI locally, but we just await here
            const count = await SecurityService.checkBreach(item.password);
            
            update(state => {
                const current = state.items[item.id];
                if (current) {
                    state.items[item.id] = {
                        ...current,
                        isBreached: count > 0,
                        breachCount: count
                    };
                }
                return state;
            });
        }
    };
}

export const securityDashboard = createSecurityStore();
