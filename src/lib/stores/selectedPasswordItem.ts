import { writable } from 'svelte/store';
import type { PasswordItem } from '$lib/types/password';

export const selectedPasswordItemStore = writable<PasswordItem | null>(null);
