import { writable } from 'svelte/store';
import type { PasswordItem } from '../../routes/+layout.ts';

export const selectedPasswordItemStore = writable<PasswordItem | null>(null);
