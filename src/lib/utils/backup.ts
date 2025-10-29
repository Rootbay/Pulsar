import { invoke } from '@tauri-apps/api/core';
import type { PasswordItem } from '$lib/types/password';

export interface VaultBackupButton {
  id: number;
  text: string;
  icon: string;
  color: string;
}

export interface VaultBackupRecipientKey {
  id: number;
  name: string;
  public_key: string;
  private_key: string;
}

export interface VaultBackupSnapshot {
  version: number;
  exportedAt: string;
  passwordItems: PasswordItem[];
  buttons: VaultBackupButton[];
  recipientKeys: VaultBackupRecipientKey[];
}

export type ImportVaultProgressStage = 'decrypting' | 'restoring';

export interface ImportVaultBackupOptions {
  sourcePath?: string | null;
  onProgress?: (stage: ImportVaultProgressStage) => void;
}

export async function buildVaultSnapshot(): Promise<VaultBackupSnapshot> {
  const [passwordItems, buttons, recipientKeys] = await Promise.all([
    invoke<PasswordItem[]>('get_password_items'),
    invoke<VaultBackupButton[]>('get_buttons'),
    invoke<VaultBackupRecipientKey[]>('get_recipient_keys')
  ]);

  return {
    version: 1,
    exportedAt: new Date().toISOString(),
    passwordItems,
    buttons,
    recipientKeys
  };
}

export async function exportVaultBackup(
  passphrase: string,
  options: { plaintext?: boolean } = {}
): Promise<string> {
  const snapshot = await buildVaultSnapshot();
  const vaultData = JSON.stringify(snapshot);

  return invoke<string>('export_vault', {
    vaultData,
    passphrase,
    isPlaintext: options.plaintext ?? false
  });
}

export async function importVaultBackup(
  passphrase: string,
  options: ImportVaultBackupOptions = {}
): Promise<VaultBackupSnapshot> {
  const { sourcePath = null, onProgress } = options;

  onProgress?.('decrypting');
  const decrypted = await invoke<string>('import_vault', {
    passphrase,
    path: sourcePath
  });

  let snapshot: VaultBackupSnapshot;
  try {
    snapshot = JSON.parse(decrypted) as VaultBackupSnapshot;
  } catch (error) {
    console.error('Failed to parse imported vault snapshot:', error);
    throw new Error('The imported vault could not be parsed.');
  }

  onProgress?.('restoring');
  await invoke('restore_vault_snapshot', { snapshot });

  return snapshot;
}

export function notifyVaultRefresh(reason: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.dispatchEvent(
    new CustomEvent('pulsar:reload-vault', {
      detail: { reason }
    })
  );
}
