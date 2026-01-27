import { callBackend } from '$lib/utils/backend';
import { pruneRecentDatabases, addRecentDatabase } from '$lib/stores/recentDatabases.svelte';
import { initAppSettings, settings } from '$lib/stores/appSettings.svelte';
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
      await addRecentDatabase(activePath);
    }
  } catch (e) {
    console.warn('Could not read active DB path from backend:', e);
  }

  const recentDbPaths = settings.state.recentDatabases;
  const defaultVault = settings.state.general.defaultVaultOnStartup;

  if (defaultVault !== 'none' && recentDbPaths.length > 0 && !activePath) {
    // If defaultVault is 'last_used' or a specific path, we use it.
    // For now, we only support 'last_used' or 'none'.
    const targetPath = defaultVault === 'last_used' ? recentDbPaths[0] : defaultVault;

    try {
      if (targetPath && targetPath !== 'last_used' && targetPath !== 'none') {
        await callBackend('switch_database', { dbPath: targetPath });
        activePath = targetPath;
      } else if (defaultVault === 'last_used') {
        const latestDbPath = recentDbPaths[0];
        await callBackend('switch_database', { dbPath: latestDbPath });
        activePath = latestDbPath;
      }
    } catch (e) {
      console.error(`Failed to switch to target database ${targetPath}:`, e);
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
