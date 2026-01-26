import { callBackend } from '$lib/utils/backend';
import { get } from 'svelte/store';
import { recentDatabases, pruneRecentDatabases } from '$lib/stores/recentDatabases';
import { initAppSettings } from '$lib/stores/appSettings.svelte';
import { appState } from '$lib/stores';

export const ssr = false;

export type { PasswordItem } from '$lib/types/password';

export async function load() {
  await initAppSettings();
  await pruneRecentDatabases();

  let activePath: string | null = null;
  try {
    activePath = await callBackend<string | null>('get_active_db_path');
    if (activePath) {
      await recentDatabases.addRecentDatabase(activePath);
    }
  } catch (e) {
    console.warn('Could not read active DB path from backend:', e);
  }

  const recentDbPaths = get(recentDatabases);
  if (recentDbPaths.length > 0 && !activePath) {
    const latestDbPath = recentDbPaths[0];
    try {
      await callBackend('switch_database', { dbPath: latestDbPath });
      activePath = latestDbPath;
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

  if (activePath) {
    appState.isDatabaseLoaded = true;
    try {
      const isConfigured = await callBackend<boolean>('is_master_password_configured');
      appState.needsPasswordSetup = !isConfigured;
    } catch (e) {
      console.error('Failed to check if master password is configured:', e);
    }
    appState.isLocked = vaultLocked;
  }

  return { vaultLocked };
}
