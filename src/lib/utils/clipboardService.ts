import { get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

import type { ClipboardSettings } from '$lib/config/settings';
import { clipboardSettings } from '$lib/stores/clipboard';

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

async function applyPolicy(settings: ClipboardSettings): Promise<ClipboardPolicyStatus> {
  return invoke<ClipboardPolicyStatus>('apply_clipboard_policy', {
    payload: {
      clipboardIntegration: settings.clipboardIntegration,
      blockHistory: settings.blockHistory,
      onlyUnlocked: settings.onlyUnlocked
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

  const settings = get(clipboardSettings);

  try {
    const status = await applyPolicy(settings);
    updateStateFromStatus(status);
  } catch (error) {
    const message = extractErrorMessage(error);
    clipboardIntegrationState.update((current) => ({
      ...current,
      lastError: message
    }));

    if (settings.blockHistory && message.toLowerCase().includes('not supported')) {
      const sanitized: ClipboardSettings = { ...settings, blockHistory: false };
      clipboardSettings.set(sanitized);
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

export async function updateClipboardSettings(
  partial: Partial<ClipboardSettings>
): Promise<void> {
  const current = get(clipboardSettings);
  const next: ClipboardSettings = { ...current, ...partial };

  clipboardIntegrationState.update((state) => ({ ...state, applying: true, lastError: null }));

  try {
    const status = await applyPolicy(next);
    clipboardSettings.set(next);
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
