import type { PasswordItem } from '$lib/types/password';
import { clear, readText, writeText } from '@tauri-apps/plugin-clipboard-manager';
import { get } from 'svelte/store';

import { appState } from '$lib/stores';
import { clipboardSettings } from '$lib/stores/clipboard';
import { clipboardIntegrationState, clipboardServiceReady } from '$lib/utils/clipboardService';

let clearTimer: ReturnType<typeof setTimeout> | null = null;

async function copyToClipboard(text: string) {
  const settings = get(clipboardSettings);
  const integrationStatus = get(clipboardIntegrationState);
  const serviceReady = get(clipboardServiceReady);
  const locked = appState.isLocked;

  if (!settings.clipboardIntegration || !integrationStatus.integrationAvailable) {
    throw new Error('Clipboard integration is disabled.');
  }

  if (settings.onlyUnlocked && locked) {
    throw new Error('Clipboard access is disabled while the vault is locked.');
  }

  if (settings.blockHistory && serviceReady) {
    if (!integrationStatus.historyBlockingSupported) {
      throw new Error('Clipboard history blocking is not supported on this platform.');
    }

    if (!integrationStatus.historyBlockingActive) {
      throw new Error('Clipboard history blocking could not be enforced.');
    }
  }

  await writeText(text);

  if (clearTimer) {
    clearTimeout(clearTimer);
  }

  if (settings.clearAfterDuration > 0) {
    const delayMs = settings.clearAfterDuration * 1000;
    clearTimer = setTimeout(async () => {
      try {
        const currentClipboard = await readText();
        if (currentClipboard === text) {
          await clear();
        }
      } catch (error) {
        console.error('Failed to clear clipboard after timeout', error);
      } finally {
        clearTimer = null;
      }
    }, delayMs);
  }
}

export async function copyPassword(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.password) {
    await copyToClipboard(passwordItem.password);
  }
}

export async function copyUsername(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.username) {
    await copyToClipboard(passwordItem.username);
  }
}

export async function copyUrl(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.url) {
    await copyToClipboard(passwordItem.url);
  }
}

export async function copyTitle(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.title) {
    await copyToClipboard(passwordItem.title);
  }
}

export async function copyText(value: string | null | undefined) {
  if (!value) {
    return;
  }
  await copyToClipboard(value);
}
