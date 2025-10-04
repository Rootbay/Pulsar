import { invoke } from "@tauri-apps/api/core";
import { get } from 'svelte/store';
import { recentDatabases } from '$lib/stores/recentDatabases';
import { initAppSettings } from '$lib/stores/appSettings';
import type { PasswordItem } from '$lib/types/password';

export const ssr = false;

export type { PasswordItem } from '$lib/types/password';

export async function load() {
    await initAppSettings();

    const recentDbPaths = get(recentDatabases);
    if (recentDbPaths.length > 0) {
        const latestDbPath = recentDbPaths[0];
        try {
            await invoke('switch_database', { dbPath: latestDbPath });
        } catch (e) {
            console.error(`Failed to switch to latest database ${latestDbPath}:`, e);
        }
    }

    let vaultLocked = true;
    try {
        vaultLocked = await invoke('is_locked');
    } catch (e) {
        console.error("Failed to check if vault is locked:", e);
    }

    return { vaultLocked };
}