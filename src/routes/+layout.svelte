<script lang="ts">
  import '../app.css';
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { appState } from '$lib/stores';
  import { settings } from '$lib/stores/appSettings.svelte';
  import { initClipboardService } from '$lib/utils/clipboardService.svelte';
  import SecurityManager from '$lib/components/SecurityManager.svelte';
  import { Toaster } from '$lib/components/ui/sonner';

  let { children } = $props();

  const AUTH_ROUTES = new Set(['/select-vault', '/setup', '/login', '/totp']);

  let prefersDark = $state(false);

  onMount(() => {
    initClipboardService().catch((error) => {
      console.error('Failed to initialize clipboard policies', error);
    });

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    prefersDark = mediaQuery.matches;

    const handler = (e: MediaQueryListEvent) => {
      prefersDark = e.matches;
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  });

  $effect(() => {
    if (browser && settings.isInitialized) {
      const htmlElement = document.documentElement;
      const currentTheme = settings.state.appearance.theme;

      htmlElement.classList.remove('light', 'dark');

      if (currentTheme === 'system') {
        htmlElement.classList.add(prefersDark ? 'dark' : 'light');
      } else {
        htmlElement.classList.add(currentTheme);
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
