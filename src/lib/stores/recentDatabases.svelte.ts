import { callBackend } from '../utils/backend';
import { settings } from './appSettings.svelte';

async function filterNonExistentDatabases(paths: string[]): Promise<string[]> {
  const existentPaths: string[] = [];
  for (const path of paths) {
    try {
      const exists = await callBackend<boolean>('check_file_exists', { path });
      if (exists) {
        existentPaths.push(path);
      } else {
        console.warn(`Non-existent database path removed from recent list: ${path}`);
      }
    } catch (e) {
      console.warn(`Skipping existence check failure for ${path}; keeping in recent list.`, e);
      existentPaths.push(path);
    }
  }
  return existentPaths;
}

export async function addRecentDatabase(path: string) {
  try {
    void callBackend('check_file_exists', { path })
      .then((exists) => {
        if (!exists) {
          console.warn(`Recent path does not yet exist (will be trimmed later): ${path}`);
        }
      })
      .catch(() => {});
  } catch {
    // ignore
  }

  const current = settings.state.recentDatabases || [];
  const filteredPaths = current.filter((p) => p !== path);
  settings.state.recentDatabases = [path, ...filteredPaths].slice(0, 5);
  settings.save();
}

export function removeRecentDatabase(path: string) {
  const current = settings.state.recentDatabases || [];
  settings.state.recentDatabases = current.filter((p) => p !== path);
  settings.save();
}

export function clearRecentDatabases() {
  settings.state.recentDatabases = [];
  settings.save();
}

export async function pruneRecentDatabases() {
  const current = settings.state.recentDatabases || [];
  const filtered = await filterNonExistentDatabases(current);
  settings.state.recentDatabases = filtered;
  settings.save();
}
