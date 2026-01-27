import { callBackend } from './backend';
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

export interface VaultBackupAttachment {
  id: number;
  itemId: number;
  fileName: string;
  fileSize: number;
  mimeType: string;
  createdAt: string;
  dataB64: string;
}

export interface VaultBackupSnapshot {
  version: number;
  exportedAt: string;
  passwordItems: PasswordItem[];
  buttons: VaultBackupButton[];
  recipientKeys: VaultBackupRecipientKey[];
  attachments: VaultBackupAttachment[];
}

export type ImportVaultProgressStage = 'decrypting' | 'restoring';

export interface ImportVaultBackupOptions {
  sourcePath?: string | null;
  onProgress?: (stage: ImportVaultProgressStage) => void;
}

export async function exportVaultBackup(
  passphrase: string,
  options: { plaintext?: boolean; masterPassword?: string } = {}
): Promise<string> {
  return callBackend<string>('export_vault_backend', {
    passphrase,
    isPlaintext: options.plaintext ?? false,
    reauthPassword: options.masterPassword ?? ''
  });
}

export async function importVaultBackup(
  passphrase: string,
  options: ImportVaultBackupOptions & { masterPassword?: string } = {}
): Promise<VaultBackupSnapshot> {
  const { sourcePath = null, onProgress, masterPassword = '' } = options;

  onProgress?.('decrypting');
  const snapshot = await callBackend<VaultBackupSnapshot>('restore_vault_backend', {
    passphrase,
    path: sourcePath,
    reauthPassword: masterPassword
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
