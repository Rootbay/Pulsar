<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { securitySettings } from '$lib/stores/security';
  import { isLocked, totpVerified } from '$lib/stores';
  import { callBackend } from '$lib/utils/backend';
  import { get } from 'svelte/store';

  let inactivityTimer: ReturnType<typeof setTimeout> | null = null;
  let clipboardTimer: ReturnType<typeof setTimeout> | null = null;

  function parseDuration(duration: string): number {
    const parts = duration.split(' ');
    const value = parseInt(parts[0]);
    const unit = parts[1]?.toLowerCase();

    if (isNaN(value)) return 0;

    switch (unit) {
      case 'minute':
      case 'minutes':
        return value * 60 * 1000;
      case 'hour':
      case 'hours':
        return value * 60 * 60 * 1000;
      default:
        return 0;
    }
  }

  async function lockVault() {
    if (get(isLocked)) return;
    try {
      await callBackend('lock');
      isLocked.set(true);
      totpVerified.set(false);
      console.log('Vault auto-locked due to inactivity');
    } catch (error) {
      console.error('Auto-lock failed:', error);
    }
  }

  async function clearClipboard() {
    try {
      await callBackend('clear_clipboard');
      console.log('Clipboard cleared automatically');
    } catch (error) {
      console.error('Failed to clear clipboard:', error);
    }
  }

  function resetInactivityTimer() {
    if (inactivityTimer) clearTimeout(inactivityTimer);

    const settings = get(securitySettings);
    if (settings.autoLockInactivity === 'Never' || get(isLocked)) {
      return;
    }

    const duration = parseDuration(settings.autoLockInactivity);
    if (duration > 0) {
      inactivityTimer = setTimeout(lockVault, duration);
    }
  }

  function handleActivity() {
    resetInactivityTimer();
  }

  // TODO: Integrate with actual copy events if possible
  function startClipboardTimer() {
    if (clipboardTimer) clearTimeout(clipboardTimer);

    const settings = get(securitySettings);
    if (settings.clearClipboardOnCopy && settings.clipboardClearTime > 0) {
    }
  }

  onMount(() => {
    window.addEventListener('mousemove', handleActivity);
    window.addEventListener('keydown', handleActivity);
    window.addEventListener('mousedown', handleActivity);
    window.addEventListener('touchstart', handleActivity);

    resetInactivityTimer();

    return () => {
      window.removeEventListener('mousemove', handleActivity);
      window.removeEventListener('keydown', handleActivity);
      window.removeEventListener('mousedown', handleActivity);
      window.removeEventListener('touchstart', handleActivity);
      if (inactivityTimer) clearTimeout(inactivityTimer);
      if (clipboardTimer) clearTimeout(clipboardTimer);
    };
  });

  $effect(() => {
    const locked = $isLocked;
    const settings = $securitySettings;
    if (!locked) {
      resetInactivityTimer();
    } else {
      if (inactivityTimer) clearTimeout(inactivityTimer);
    }
  });
</script>
