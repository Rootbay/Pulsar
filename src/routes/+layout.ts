import { callBackend } from '$lib/utils/backend';
import { get } from 'svelte/store';
import { recentDatabases, pruneRecentDatabases } from '$lib/stores/recentDatabases';
import { initAppSettings } from '$lib/stores/appSettings.svelte';
import type { PasswordItem } from '$lib/types/password';

export const ssr = false;

export type { PasswordItem } from '$lib/types/password';

export async function load() {
  await initAppSettings();
  await pruneRecentDatabases();

  try {
    const activePath = await callBackend<string | null>('get_active_db_path');
    if (activePath) {
      await recentDatabases.addRecentDatabase(activePath);
    }
  } catch (e) {
    console.warn('Could not read active DB path from backend:', e);
  }

  const recentDbPaths = get(recentDatabases);
  if (recentDbPaths.length > 0) {
    const latestDbPath = recentDbPaths[0];
    try {
      await callBackend('switch_database', { dbPath: latestDbPath });
    } catch (e) {
      console.error(`Failed to switch to latest database ${latestDbPath}:`, e);
    }
  }

  let vaultLocked = true;
  try {
    vaultLocked = await callBackend('is_locked');
  } catch (e) {
    console.error('Failed to check if vault is locked:', e);
  }

  return { vaultLocked };
}
