import { derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { appSettings } from './appSettings';

async function filterNonExistentDatabases(paths: string[]): Promise<string[]> {
    const existentPaths: string[] = [];
    for (const path of paths) {
        try {
            const exists = await invoke('check_file_exists', { path });
            if (exists) {
                existentPaths.push(path);
            } else {
                console.warn(`Non-existent database path removed from recent list: ${path}`);
            }
        } catch (e) {
            console.error(`Error checking file existence for ${path}:`, e);
        }
    }
    return existentPaths;
}

function createRecentDatabasesStore() {
    const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.recentDatabases);

    return {
        subscribe,
        addRecentDatabase: async (path: string) => {
            try {
                const exists = await invoke('check_file_exists', { path });
                if (!exists) {
                    console.warn(`Attempted to add non-existent database path: ${path}`);
                    return;
                }
            } catch (e) {
                console.error(`Error checking file existence for ${path} before adding:`, e);
                return;
            }

            appSettings.update((settings) => {
                const filteredPaths = settings.recentDatabases.filter((p) => p !== path);
                settings.recentDatabases = [path, ...filteredPaths].slice(0, 5);
                return settings;
            });
        },
        removeRecentDatabase: (path: string) => {
            appSettings.update((settings) => {
                settings.recentDatabases = settings.recentDatabases.filter((p) => p !== path);
                return settings;
            });
        },
        clearRecentDatabases: () => {
            appSettings.update((settings) => {
                settings.recentDatabases = [];
                return settings;
            });
        },
    };
}

export const recentDatabases = createRecentDatabasesStore();

// Initialize recent databases by filtering out non-existent ones
async function initializeRecentDatabases() {
    appSettings.update((settings) => {
        filterNonExistentDatabases(settings.recentDatabases).then(filteredPaths => {
            appSettings.update(currentSettings => {
                return {
                    ...currentSettings,
                    recentDatabases: filteredPaths,
                };
            });
        });
        return settings;
    });
}

initializeRecentDatabases();
