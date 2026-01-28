import { i18n } from '$lib/i18n.svelte';

/**
 * Parses an unknown error into a human-readable string.
 * Supports strings, Error objects, and Tauri-style error objects with message/code.
 */
export function parseError(error: unknown): string {
  if (!error) return '';

  const locale = i18n.locale;

  if (typeof error === 'string') {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'object') {
    const e = error as Record<string, unknown>;
    if (typeof e.message === 'string') {
      return e.message;
    }
    try {
      return JSON.stringify(error);
    } catch {
      // ignore
    }
  }

  return locale === 'sv' ? 'Ett oväntat fel inträffade.' : 'An unexpected error occurred.';
}

/**
 * Legacy alias for parseError used in some components.
 */
export const toErrorMessage = parseError;
