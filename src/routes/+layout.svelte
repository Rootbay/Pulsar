<script lang="ts">
  import '../app.css';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import {
    isDatabaseLoaded,
    needsPasswordSetup,
    isLocked,
    totpVerified,
    totpRequired
  } from '$lib/stores';
  import { appearanceSettings } from '$lib/stores/appearance';
  import { initClipboardService } from '$lib/utils/clipboardService';

  let { children } = $props();

  const AUTH_ROUTES = new Set(['/select-vault', '/setup', '/login', '/totp']);

  onMount(() => {
    initClipboardService().catch((error) => {
      console.error('Failed to initialize clipboard policies', error);
    });
  });

  $effect(() => {
    if (browser) {
      const htmlElement = document.documentElement;
      const currentTheme = $appearanceSettings.theme;

      htmlElement.classList.remove('theme-light', 'theme-dark', 'theme-system');

      if (currentTheme === 'system') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        htmlElement.classList.add(prefersDark ? 'theme-dark' : 'theme-light');
      } else {
        htmlElement.classList.add(`theme-${currentTheme}`);
      }
    }
  });

  const needsTotp = $derived(
    $totpRequired && !$totpVerified && $isDatabaseLoaded && !$isLocked && !$needsPasswordSetup
  );

  const requiredAuthRoute = $derived(
    !$isDatabaseLoaded
      ? '/select-vault'
      : $needsPasswordSetup
        ? '/setup'
        : $isLocked
          ? '/login'
          : needsTotp
            ? '/totp'
            : null
  );

  $effect(() => {
    if (browser) {
      const currentPath = $page.url.pathname;

      if (requiredAuthRoute) {
        if (currentPath !== requiredAuthRoute) {
          goto(requiredAuthRoute, { replaceState: true });
        }
      } else if (AUTH_ROUTES.has(currentPath)) {
        goto('/', { replaceState: true });
      }
    }
  });
</script>

{@render children()}
