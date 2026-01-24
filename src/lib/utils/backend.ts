import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';

export async function callBackend<T>(
  command: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: any) {
    let errorMessage = 'An unknown error occurred';
    let errorCode = 'Unknown';

    if (typeof error === 'string') {
      errorMessage = error;
    } else if (error && typeof error === 'object') {
      errorMessage = error.message || JSON.stringify(error);
      errorCode = error.code || 'Unknown';
    }

    console.error(`Backend error in ${command} [${errorCode}]:`, errorMessage);

    if (!command.startsWith('is_')) {
      toast.error(errorMessage);
    }

    throw error;
  }
}

export enum BackendErrorCode {
  Database = 'Database',
  VaultLocked = 'VaultLocked',
  VaultNotLoaded = 'VaultNotLoaded',
  InvalidPassword = 'InvalidPassword',
  Validation = 'Validation',
  Internal = 'Internal'
}

export interface BackendError {
  code: BackendErrorCode;
  message: string;
}
