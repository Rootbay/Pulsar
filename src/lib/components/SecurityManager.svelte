<script lang="ts">
  import { onMount } from 'svelte';
  import { settings } from '$lib/stores/appSettings.svelte';
  import { appState } from '$lib/stores';
  import { callBackend } from '$lib/utils/backend';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { SecuritySettings } from '$lib/config/settings';

  let inactivityTimer: ReturnType<typeof setTimeout> | null = null;
  let minimizeTimer: ReturnType<typeof setTimeout> | null = null;
  let suspendTimer: ReturnType<typeof setTimeout> | null = null;

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
    if (appState.isLocked) return;
    try {
      await callBackend('lock');
      appState.isLocked = true;
      appState.totpVerified = false;
      console.log('Vault auto-locked due to inactivity');
    } catch (error) {
      console.error('Auto-lock failed:', error);
    }
  }

  function resetInactivityTimer() {
    if (inactivityTimer) clearTimeout(inactivityTimer);

    const currentSettings = settings.state.security;
    if (currentSettings.autoLockInactivity === 'Never' || appState.isLocked) {
      return;
    }

    const duration = parseDuration(currentSettings.autoLockInactivity);
    if (duration > 0) {
      inactivityTimer = setTimeout(lockVault, duration);
    }
  }

  function clearMinimizeTimer() {
    if (minimizeTimer) {
      clearTimeout(minimizeTimer);
      minimizeTimer = null;
    }
  }

  function clearSuspendTimer() {
    if (suspendTimer) {
      clearTimeout(suspendTimer);
      suspendTimer = null;
    }
  }

  function getLockGraceMs(currentSettings: SecuritySettings) {
    const seconds = Number.isFinite(currentSettings.lockGraceSeconds)
      ? Math.max(0, Math.min(60, currentSettings.lockGraceSeconds))
      : 0;
    return seconds * 1000;
  }

  function handleActivity() {
    resetInactivityTimer();
  }

  onMount(() => {
    const appWindow = getCurrentWindow();
    let unlistenBlur: (() => void) | null = null;

    window.addEventListener('mousemove', handleActivity);
    window.addEventListener('keydown', handleActivity);
    window.addEventListener('mousedown', handleActivity);
    window.addEventListener('touchstart', handleActivity);
    resetInactivityTimer();

    appWindow
      .onFocusChanged(async ({ payload }) => {
        if (payload === true) {
          clearMinimizeTimer();
          resetInactivityTimer();
          return;
        }
        if (payload === false) {
          const currentSettings = settings.state.security;
          if (currentSettings.lockOnMinimize) {
            try {
              const minimized = await appWindow.isMinimized();
              if (minimized) {
                clearMinimizeTimer();
                const graceMs = getLockGraceMs(currentSettings);
                if (graceMs === 0) {
                  await lockVault();
                } else {
                  minimizeTimer = setTimeout(async () => {
                    await lockVault();
                    minimizeTimer = null;
                  }, graceMs);
                }
              }
            } catch (error) {
              console.warn('Failed to read window minimize state', error);
            }
          }
        }
      })
      .then((unlisten) => {
        unlistenBlur = unlisten;
      })
      .catch((error) => {
        console.warn('Failed to register window focus listener', error);
      });

    const handleVisibility = async () => {
      if (!document.hidden) {
        clearSuspendTimer();
        resetInactivityTimer();
        return;
      }
      const currentSettings = settings.state.security;
      if (currentSettings.lockOnSuspend) {
        clearSuspendTimer();
        const graceMs = getLockGraceMs(currentSettings);
        if (graceMs === 0) {
          await lockVault();
        } else {
          suspendTimer = setTimeout(async () => {
            await lockVault();
            suspendTimer = null;
          }, graceMs);
        }
      }
    };

    document.addEventListener('visibilitychange', handleVisibility);

    return () => {
      window.removeEventListener('mousemove', handleActivity);
      window.removeEventListener('keydown', handleActivity);
      window.removeEventListener('mousedown', handleActivity);
      window.removeEventListener('touchstart', handleActivity);
      document.removeEventListener('visibilitychange', handleVisibility);
      if (unlistenBlur) {
        unlistenBlur();
      }
      clearMinimizeTimer();
      clearSuspendTimer();
      if (inactivityTimer) clearTimeout(inactivityTimer);
    };
  });

  $effect(() => {
    const locked = appState.isLocked;
    const currentSettings = settings.state.security;
    void currentSettings.autoLockInactivity;
    void currentSettings.lockGraceSeconds;

    if (!locked) {
      resetInactivityTimer();
    } else {
      if (inactivityTimer) clearTimeout(inactivityTimer);
    }
  });
</script>
