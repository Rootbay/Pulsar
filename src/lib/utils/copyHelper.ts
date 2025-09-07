import type { PasswordItem } from '../../routes/+layout.ts';
import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';
import { securitySettings } from '$lib/stores/security';
import { get } from 'svelte/store';

async function copyToClipboard(text: string) {
    await writeText(text);

    const settings = get(securitySettings);

    if (settings.clearClipboardOnCopy && settings.clipboardClearTime > 0) {
        setTimeout(async () => {
            const currentClipboard = await readText();
            if (currentClipboard === text) {
                await writeText('');
            }
        }, settings.clipboardClearTime * 1000);
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
