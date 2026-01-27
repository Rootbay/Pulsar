import type { PasswordItem } from '$lib/types/password';
import { clear, readText, writeText } from '@tauri-apps/plugin-clipboard-manager';

import { appState } from '$lib/stores';
import { settings } from '$lib/stores/appSettings.svelte';
import { clipboardService } from '$lib/utils/clipboardService.svelte';

async function copyToClipboard(text: string, label: string = 'Text') {
  const clipSettings = settings.state.clipboard;
  const integrationStatus = clipboardService.state;
  const serviceReady = clipboardService.ready;
  const locked = appState.isLocked;
  if (!clipSettings.clipboardIntegration || !integrationStatus.integrationAvailable) {
    throw new Error('Clipboard integration is disabled.');
  }

  if (clipSettings.onlyUnlocked && locked) {
    throw new Error('Clipboard access is disabled while the vault is locked.');
  }

  if (clipSettings.blockHistory && serviceReady) {
    if (!integrationStatus.historyBlockingSupported) {
      throw new Error('Clipboard history blocking is not supported on this platform.');
    }

    if (!integrationStatus.historyBlockingActive) {
      throw new Error('Clipboard history blocking could not be enforced.');
    }
  }

  await writeText(text);
  await clipboardService.recordCopy(text, label);
}

export async function copyPassword(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.password) {
    await copyToClipboard(passwordItem.password, 'Password');
  }
}

export async function copyUsername(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.username) {
    await copyToClipboard(passwordItem.username, 'Username');
  }
}

export async function copyUrl(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.url) {
    await copyToClipboard(passwordItem.url, 'URL');
  }
}

export async function copyTitle(passwordItem: PasswordItem) {
  if (passwordItem && passwordItem.title) {
    await copyToClipboard(passwordItem.title, 'Title');
  }
}

export async function copyText(value: string | null | undefined, label: string = 'Text') {
  if (!value) {
    return;
  }
  await copyToClipboard(value, label);
}
