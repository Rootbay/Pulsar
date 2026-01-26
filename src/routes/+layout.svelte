<script lang="ts">
  import '../app.css';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { appState } from '$lib/stores';
  import { appearanceSettings } from '$lib/stores/appearance';
  import { initClipboardService } from '$lib/utils/clipboardService';
  import SecurityManager from '$lib/components/SecurityManager.svelte';
  import { Toaster } from '$lib/components/ui/sonner';

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

  const requiredAuthRoute = $derived(
    !appState.isDatabaseLoaded
      ? '/select-vault'
      : appState.needsPasswordSetup
        ? '/setup'
        : appState.isLocked
          ? '/login'
          : appState.totpRequired && !appState.totpVerified
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

<SecurityManager />
<Toaster />

{@render children()}
