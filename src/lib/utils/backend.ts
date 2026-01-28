export async function callBackend<T>(
  command: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    let errorMessage = 'An unknown error occurred';
    let errorCode = 'Unknown';

    if (typeof error === 'string') {
      errorMessage = error;
    } else if (error && typeof error === 'object') {
      const e = error as Record<string, unknown>;
      errorMessage = (e.message as string) || JSON.stringify(error);
      errorCode = (e.code as string) || 'Unknown';
    }

    console.error(`Backend error in ${command} [${errorCode}]:`, errorMessage);

    const isSilent =
      command.startsWith('is_') ||
      command.startsWith('check_') ||
      errorMessage.toLowerCase().includes('cancel');

    if (!isSilent) {
      import('svelte-sonner').then(({ toast }) => {
        toast.error(errorMessage);
      }).catch(() => {});
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