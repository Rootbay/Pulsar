import { derived, get } from 'svelte/store';
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
            // If the check fails (e.g., Tauri not ready), keep the entry to avoid wiping the list.
            console.warn(`Skipping existence check failure for ${path}; keeping in recent list.`, e);
            existentPaths.push(path);
        }
    }
    return existentPaths;
}

function createRecentDatabasesStore() {
    const { subscribe } = derived(appSettings, ($appSettings) => {
        const arr = Array.isArray($appSettings.recentDatabases) ? $appSettings.recentDatabases : [];
        // Deduplicate while preserving order
        const seen = new Set<string>();
        const unique: string[] = [];
        for (const p of arr) {
            if (!seen.has(p)) {
                seen.add(p);
                unique.push(p);
            }
        }
        return unique;
    });

    return {
        subscribe,
        addRecentDatabase: async (path: string) => {
            // Accept the path immediately so UI updates; existence is re-validated on startup and selection
            // (some flows create the file only after initial configuration)
            try {
                // Fire-and-forget existence check to trim later if needed
                void invoke('check_file_exists', { path }).then((exists) => {
                    if (!exists) {
                        console.warn(`Recent path does not yet exist (will be trimmed later): ${path}`);
                    }
                }).catch(() => {});
            } catch {
                // ignore
            }

            appSettings.update((settings) => {
                const filteredPaths = settings.recentDatabases.filter((p) => p !== path);
                return {
                    ...settings,
                    recentDatabases: [path, ...filteredPaths].slice(0, 5)
                };
            });
            // Persist immediately so a quick refresh doesn't lose the entry
            try {
                await invoke('set_all_settings', { settings: JSON.stringify(get(appSettings)) });
            } catch (e) {
                console.error('Failed to persist recent databases immediately:', e);
            }
        },
        removeRecentDatabase: (path: string) => {
            appSettings.update((settings) => ({
                ...settings,
                recentDatabases: settings.recentDatabases.filter((p) => p !== path)
            }));
            void (async () => {
                try {
                    await invoke('set_all_settings', { settings: JSON.stringify(get(appSettings)) });
                } catch (e) {
                    console.error('Failed to persist recent databases after remove:', e);
                }
            })();
        },
        clearRecentDatabases: () => {
            appSettings.update((settings) => ({
                ...settings,
                recentDatabases: []
            }));
            void (async () => {
                try {
                    await invoke('set_all_settings', { settings: JSON.stringify(get(appSettings)) });
                } catch (e) {
                    console.error('Failed to persist recent databases after clear:', e);
                }
            })();
        },
    };
}

export const recentDatabases = createRecentDatabasesStore();

// Expose a safe, explicit initializer to prune non-existent entries once the app settings are loaded.
export async function pruneRecentDatabases() {
    const currentSettings = get(appSettings);
    const filtered = await filterNonExistentDatabases(currentSettings.recentDatabases);
    appSettings.update((settings) => ({
        ...settings,
        recentDatabases: filtered,
    }));
}
