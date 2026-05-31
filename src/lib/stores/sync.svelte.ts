import { callBackend } from '../utils/backend';
import { settings } from './appSettings.svelte';
import { appState } from './appState.svelte';
import { toast } from '$lib/components/ui/sonner';
import { notifyVaultRefresh } from '$lib/utils/backup';
import type { VaultBackupSnapshot } from '$lib/utils/backup';
import { parseError } from '$lib/utils/error';

class SyncStore {
  isSyncing = $state(false);
  lastSyncTime = $state('');
  syncError = $state<string | null>(null);

  async performSync(force: boolean = false) {
    if (this.isSyncing) return;

    if (appState.isLocked || !appState.isDatabaseLoaded) {
      return;
    }

    const backup = settings.state.backup;
    const { webdavUrl, webdavUsername, webdavPassword, syncMode } = backup;

    if (!force && syncMode === 'off') {
      return;
    }

    if (!webdavUrl || !webdavUsername || !webdavPassword) {
      this.syncError = 'WebDAV configuration is incomplete.';
      return;
    }

    this.isSyncing = true;
    this.syncError = null;

    try {
      const authHeader = 'Basic ' + btoa(webdavUsername + ':' + webdavPassword);
      const baseUrl = webdavUrl.replace(/\/$/, '');
      const folder = backup.webdavSyncFolder || '/PulsarSync';
      const cleanFolder = '/' + folder.replace(/^\/|\/$/g, '');
      const folderUrl = `${baseUrl}${cleanFolder}`;
      const fileUrl = `${folderUrl}/vault.pulsarsync`;

      // 1. Check/Create Remote Directory
      let folderExists = false;
      try {
        const res = await fetch(folderUrl, {
          method: 'PROPFIND',
          headers: {
            Authorization: authHeader,
            Depth: '0'
          }
        });
        if (res.status === 207 || res.status === 200) {
          folderExists = true;
        }
      } catch (e) {
        console.warn('Directory check failed, attempting MKCOL:', e);
      }

      if (!folderExists) {
        const mkcolRes = await fetch(folderUrl, {
          method: 'MKCOL',
          headers: {
            Authorization: authHeader
          }
        });
        if (mkcolRes.status !== 201 && mkcolRes.status !== 200) {
          throw new Error(`Failed to create remote directory: ${mkcolRes.statusText}`);
        }
      }

      // 2. Fetch Remote Payload if it exists
      let remotePayload: string | null = null;
      try {
        const getRes = await fetch(fileUrl, {
          method: 'GET',
          headers: {
            Authorization: authHeader
          }
        });
        if (getRes.status === 200) {
          remotePayload = await getRes.text();
        }
      } catch (e) {
        console.log('Sync file not found or failed to fetch, starting fresh:', e);
      }

      // 3. Dump Local Snapshot
      const localSnapshot = await callBackend<VaultBackupSnapshot>('get_vault_snapshot');

      let mergedSnapshot = localSnapshot;

      // 4. Decrypt & Merge Remote Snapshot
      if (remotePayload) {
        try {
          const remoteSnapshot = await callBackend<VaultBackupSnapshot>('decrypt_sync_payload', {
            payload: remotePayload,
            passphrase: webdavPassword
          });

          mergedSnapshot = await callBackend<VaultBackupSnapshot>('merge_vault_snapshots', {
            local: localSnapshot,
            remote: remoteSnapshot
          });
        } catch (err) {
          console.error('Remote decryption/merge failed, overwriting remote:', err);
          toast.warning('Remote decryption failed. Initializing remote with local state.');
        }
      }

      // 5. Encrypt Merged Snapshot
      const encryptedPayload = await callBackend<string>('encrypt_sync_payload', {
        snapshot: mergedSnapshot,
        passphrase: webdavPassword
      });

      // 6. Upload Encrypted Sync File
      const putRes = await fetch(fileUrl, {
        method: 'PUT',
        headers: {
          Authorization: authHeader,
          'Content-Type': 'application/json'
        },
        body: encryptedPayload
      });

      if (putRes.status !== 201 && putRes.status !== 200 && putRes.status !== 204) {
        throw new Error(`Failed to upload sync file: ${putRes.statusText}`);
      }

      // 7. Apply Merged Snapshot Locally
      await callBackend('restore_vault_snapshot', {
        snapshot: mergedSnapshot
      });

      this.lastSyncTime = new Date().toLocaleTimeString();
      backup.webdavSyncLastTime = this.lastSyncTime;
      settings.save();

      notifyVaultRefresh('sync');
      toast.success('Synchronization Completed Successfully.');
    } catch (error: unknown) {
      console.error('Sync failed:', error);
      this.syncError = parseError(error) || 'Sync failed.';
      toast.error(`Sync Failed: ${this.syncError}`);
    } finally {
      this.isSyncing = false;
    }
  }
}

export const syncStore = new SyncStore();
