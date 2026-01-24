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

export async function exportVaultBackup(
  passphrase: string,
  options: { plaintext?: boolean } = {}
): Promise<string> {
  return invoke<string>('export_vault_backend', {
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
  // restore_vault_backend handles both decryption AND restoration in one go efficiently
  const snapshot = await invoke<VaultBackupSnapshot>('restore_vault_backend', {
    passphrase,
    path: sourcePath
  });

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
