import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';

/**
 * A typed wrapper around Tauri's invoke.
 */
export async function callBackend<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
    try {
        return await invoke<T>(command, args);
    } catch (error) {
        const errorMessage = typeof error === 'string' ? error : JSON.stringify(error);
        console.error(`Backend error in ${command}:`, errorMessage);
        
        // You can customize error handling here, e.g., showing a toast
        if (!command.startsWith('is_')) { // Don't toast for check commands
            toast.error(`Error: ${errorMessage}`);
        }
        
        throw error;
    }
}

/**
 * Example of structured error handling on the frontend
 */
export enum BackendErrorCode {
    Database = 'Database',
    VaultLocked = 'VaultLocked',
    VaultNotLoaded = 'VaultNotLoaded',
    InvalidPassword = 'InvalidPassword',
    Validation = 'Validation',
    Internal = 'Internal',
}

export interface BackendError {
    code: BackendErrorCode;
    message: string;
}
