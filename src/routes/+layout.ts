import { invoke } from "@tauri-apps/api/core";
import { get } from 'svelte/store';
import { recentDatabases } from '$lib/stores/recentDatabases';
import { initAppSettings } from '$lib/stores/appSettings';

export const ssr = false;

export interface PasswordItem {
    id: number;
    title: string;
    description: string | null;
    img: string | null;
    tags: string | null;
    username: string | null;
    url: string | null;
    notes: string | null;
    password: string;
    totp_secret?: string | null;
    created_at: string;
    updated_at: string;
    color: string | null;
    custom_fields: { name: string; value: string; field_type: string }[];
    field_order?: string[] | null;
}

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