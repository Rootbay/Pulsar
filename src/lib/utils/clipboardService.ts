import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

import type { ClipboardSettings } from '$lib/config/settings';
import { settings } from '$lib/stores/appSettings.svelte';

interface ClipboardPolicyStatus {
  integrationAvailable: boolean;
  historyBlockingSupported: boolean;
  historyBlockingActive: boolean;
}

export interface ClipboardIntegrationState extends ClipboardPolicyStatus {
  applying: boolean;
  lastError: string | null;
}

const defaultStatus: ClipboardIntegrationState = {
  integrationAvailable: true,
  historyBlockingSupported: false,
  historyBlockingActive: false,
  applying: false,
  lastError: null
};

export const clipboardIntegrationState = writable<ClipboardIntegrationState>(defaultStatus);
export const clipboardServiceReady = writable(false);

let initialized = false;

function extractErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  try {
    return JSON.stringify(error);
  } catch (jsonError) {
    console.error('Failed to stringify clipboard error payload', jsonError);
    return 'Unknown clipboard error';
  }
}

function updateStateFromStatus(status: ClipboardPolicyStatus) {
  clipboardIntegrationState.update((current) => ({
    ...current,
    integrationAvailable: status.integrationAvailable,
    historyBlockingSupported: status.historyBlockingSupported,
    historyBlockingActive: status.historyBlockingActive,
    lastError: null
  }));
}

async function applyPolicy(clipboardSettings: ClipboardSettings): Promise<ClipboardPolicyStatus> {
  return invoke<ClipboardPolicyStatus>('apply_clipboard_policy', {
    payload: {
      clipboardIntegration: clipboardSettings.clipboardIntegration,
      blockHistory: clipboardSettings.blockHistory,
      onlyUnlocked: clipboardSettings.onlyUnlocked
    }
  });
}

export async function initClipboardService(): Promise<void> {
  if (initialized) return;

  try {
    const status = await invoke<ClipboardPolicyStatus>('get_clipboard_capabilities');
    updateStateFromStatus(status);
  } catch (error) {
    const message = extractErrorMessage(error);
    clipboardIntegrationState.update((current) => ({
      ...current,
      integrationAvailable: false,
      lastError: message
    }));
  }

  const currentSettings = settings.state.clipboard;

  try {
    const status = await applyPolicy(currentSettings);
    updateStateFromStatus(status);
  } catch (error) {
    const message = extractErrorMessage(error);
    clipboardIntegrationState.update((current) => ({
      ...current,
      lastError: message
    }));

    if (currentSettings.blockHistory && message.toLowerCase().includes('not supported')) {
      settings.state.clipboard.blockHistory = false;
      settings.save();
      
      const sanitized = settings.state.clipboard;
      try {
        const status = await applyPolicy(sanitized);
        updateStateFromStatus(status);
      } catch (secondaryError) {
        const secondaryMessage = extractErrorMessage(secondaryError);
        clipboardIntegrationState.update((current) => ({
          ...current,
          lastError: secondaryMessage
        }));
      }
    }
  }

  initialized = true;
  clipboardServiceReady.set(true);
}

export async function updateClipboardSettings(partial: Partial<ClipboardSettings>): Promise<void> {
  const current = settings.state.clipboard;
  const next: ClipboardSettings = { ...current, ...partial };

  clipboardIntegrationState.update((state) => ({ ...state, applying: true, lastError: null }));

  try {
    const status = await applyPolicy(next);
    settings.state.clipboard = next;
    settings.save();
    
    updateStateFromStatus(status);
  } catch (error) {
    const message = extractErrorMessage(error);
    clipboardIntegrationState.update((state) => ({ ...state, lastError: message }));
    throw new Error(message);
  } finally {
    clipboardIntegrationState.update((state) => ({ ...state, applying: false }));
  }
}

export async function clearClipboardNow(): Promise<void> {
  await invoke('clear_clipboard');
}
