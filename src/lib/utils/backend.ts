export async function callBackend<T>(
  command: string,
  args: Record<string, unknown> = {},
  retries = 3
): Promise<T> {
  let lastError: unknown;

  for (let attempt = 0; attempt < retries; attempt++) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<T>(command, args);
    } catch (error: unknown) {
      lastError = error;

      const errorMessage = typeof error === 'string' ? error : String((error as { message?: unknown })?.message || '');
      const isTransient =
        errorMessage.toLowerCase().includes('busy') ||
        errorMessage.toLowerCase().includes('timeout') ||
        errorMessage.toLowerCase().includes('locked');

      if (!isTransient || attempt === retries - 1) {
        break;
      }

      await new Promise((resolve) => setTimeout(resolve, Math.pow(2, attempt) * 100));
    }
  }

  const error = lastError;
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
    import('svelte-sonner')
      .then(({ toast }) => {
        toast.error(errorMessage);
      })
      .catch(() => { });
  }

  throw error;
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
