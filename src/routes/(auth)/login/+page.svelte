<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { callBackend } from '$lib/utils/backend';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Spinner } from '$lib/components/ui/spinner/index.js';
  import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import {
    appState
  } from '$lib/stores';
  import { currentLocale, t } from '$lib/i18n';
  import { Lock, Eye, EyeOff, ArrowLeft, FingerprintPattern, TriangleAlert } from '@lucide/svelte';
  import { onMount } from 'svelte';

  let locale = $derived($currentLocale);

  let password = $state('');
  let loginError = $state<string | null>(null);
  let isUnlocking = $state(false);
  let showPassword = $state(false);
  let isBiometricsAvailable = $state(false);
  let isBiometricUnlocking = $state(false);
  let isCapsLockOn = $state(false);

  function handleKeydown(event: KeyboardEvent) {
    if (event.getModifierState && event.getModifierState('CapsLock')) {
      isCapsLockOn = true;
    } else {
      isCapsLockOn = false;
    }
  }

  onMount(() => {
    checkBiometrics();
  });

  async function checkBiometrics() {
    try {
      isBiometricsAvailable = await callBackend<boolean>('is_biometrics_enabled');
    } catch (err) {
      console.error('Biometric check failed:', err);
    }
  }

  async function handleBiometricUnlock() {
    if (isBiometricUnlocking || isUnlocking) return;
    isBiometricUnlocking = true;
    loginError = null;

    try {
      const { authenticate } = await import('@tauri-apps/plugin-biometric');
      await authenticate(t(locale, 'loginBiometricPrompt'));

      const result = await callBackend<{ totp_required: boolean }>('unlock_with_biometrics');

      if (result?.totp_required) {
        appState.totpRequired = true;
        appState.totpVerified = false;
        appState.isLocked = false;
        await goto('/totp', { replaceState: true });
      } else {
        appState.totpRequired = false;
        appState.totpVerified = true;
        appState.isLocked = false;
        await goto('/', { replaceState: true });
      }
    } catch (error: unknown) {
      console.error('Biometric unlock failed:', error);
      const msg = (error as Record<string, unknown>).message || String(error);
      if (
        typeof msg === 'string' &&
        !msg.toLowerCase().includes('cancel') &&
        !msg.toLowerCase().includes('user canceled')
      ) {
        loginError = t(locale, 'loginBiometricFailed');
      }
    } finally {
      isBiometricUnlocking = false;
    }
  }

  $effect(() => {
    if (browser) {
      if (!appState.isDatabaseLoaded) {
        goto('/select-vault', { replaceState: true });
      } else if (appState.needsPasswordSetup) {
        goto('/setup', { replaceState: true });
      } else if (!appState.isLocked) {
        goto('/', { replaceState: true });
      }
    }
  });

  const handleUnlock = async () => {
    const trimmedPassword = password.trim();
    if (isUnlocking || !trimmedPassword) return;

    isUnlocking = true;
    loginError = null;

    try {
      const result = await callBackend<{ totp_required: boolean }>('unlock', {
        password: trimmedPassword
      });
      if (result?.totp_required) {
        appState.totpRequired = true;
        appState.totpVerified = false;
        appState.isLocked = false;
        await goto('/totp', { replaceState: true });
      } else {
        appState.totpRequired = false;
        appState.totpVerified = true;
        appState.isLocked = false;
        await goto('/', { replaceState: true });
      }
    } catch (error: unknown) {
      console.error('Unlock failed:', error);
      appState.totpRequired = false;
      appState.totpVerified = false;
      loginError =
        ((error as Record<string, unknown>).message as string) || t(locale, 'loginUnknownError');
    } finally {
      isUnlocking = false;
    }
  };

  const handleChangeDatabase = async () => {
    await callBackend('lock');
    appState.isDatabaseLoaded = false;
    appState.isLocked = true;
    appState.totpRequired = false;
    appState.totpVerified = false;
  };

  const goBack = async () => {
    await handleChangeDatabase();
    await goto('/select-vault', { replaceState: true });
  };

  const handleSubmit = (event: SubmitEvent) => {
    event.preventDefault();
    void handleUnlock();
  };

  const canSubmit = $derived(password.trim().length > 0 && !isUnlocking);
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link
    href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<div class="from-background to-background/80 relative min-h-screen bg-linear-to-b">
  <button
    type="button"
    class="text-muted-foreground hover:text-foreground absolute top-4 left-4 z-10 flex items-center gap-1 rounded-md px-2 py-1 text-sm transition"
    onclick={goBack}
  >
    <ArrowLeft class="h-4 w-4" />
    {t(locale, 'back')}
  </button>
  <div class="pointer-events-none absolute inset-0 -z-10">
    <div
      class="bg-primary/10 blob-a absolute top-[10%] left-[10%] size-112 rounded-full blur-3xl"
    ></div>
    <div
      class="bg-muted/40 blob-b absolute right-[10%] bottom-[12%] size-88 rounded-full blur-2xl"
    ></div>
  </div>

  <div class="mx-auto grid min-h-screen w-full place-items-center px-4">
    <Card
      class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 w-full max-w-md backdrop-blur"
    >
      <form class="flex flex-col" onsubmit={handleSubmit}>
        <CardHeader class="space-y-0 text-center">
          <div
            class="bg-primary/10 text-primary mx-auto mb-2 flex size-12 items-center justify-center rounded-xl"
          >
            <Lock class="size-6" />
          </div>
          <CardTitle class="text-2xl font-semibold tracking-tight">
            {t(locale, 'loginTitle')}
          </CardTitle>
          <CardDescription class="mt-0">
            {t(locale, 'loginSubtitle')}
          </CardDescription>
        </CardHeader>

        <CardContent class="mt-6 space-y-4">
          <div class="space-y-2">
            <Label for="master-password">
              {t(locale, 'loginMasterPasswordLabel')}
            </Label>
            <div class="relative">
              <Input
                id="master-password"
                type={showPassword ? 'text' : 'password'}
                placeholder={t(locale, 'loginMasterPasswordPlaceholder')}
                bind:value={password}
                autocomplete="current-password"
                disabled={isUnlocking}
                onkeydown={handleKeydown}
                class="pr-10"
              />
              <button
                type="button"
                class="text-muted-foreground hover:text-foreground absolute inset-y-0 right-0 flex items-center px-3"
                onclick={() => (showPassword = !showPassword)}
                aria-label={showPassword
                  ? t(locale, 'loginHidePassword')
                  : t(locale, 'loginShowPassword')}
                tabindex="-1"
              >
                {#if showPassword}
                  <EyeOff class="size-4" />
                {:else}
                  <Eye class="size-4" />
                {/if}
              </button>
            </div>
            {#if isCapsLockOn}
              <p
                class="text-warning-foreground flex animate-pulse items-center gap-1.5 pt-1 text-[11px] font-medium"
              >
                <TriangleAlert class="size-3" />
                {t(locale, 'loginCapsLockOn')}
              </p>
            {/if}
          </div>

          {#if loginError}
            <p class="text-destructive text-sm font-medium">{loginError}</p>
          {/if}
        </CardContent>

        <CardFooter class="mt-6 flex flex-col gap-2">
          <Button type="submit" class="w-full" disabled={!canSubmit}>
            {#if isUnlocking}
              <Spinner class="mr-2 size-4" /> {t(locale, 'loginUnlocking')}
            {:else}
              {t(locale, 'loginUnlock')}
            {/if}
          </Button>

          {#if isBiometricsAvailable}
            <Button
              type="button"
              variant="outline"
              class="w-full gap-2"
              onclick={handleBiometricUnlock}
              disabled={isBiometricUnlocking || isUnlocking}
            >
              {#if isBiometricUnlocking}
                <Spinner class="size-4" />
              {:else}
                <FingerprintPattern class="size-4" />
              {/if}
              {t(locale, 'loginUnlockBiometric')}
            </Button>
          {/if}

          <Button type="button" variant="ghost" class="w-full" onclick={handleChangeDatabase}>
            {t(locale, 'loginOpenAnotherVault')}
          </Button>
        </CardFooter>
      </form>
    </Card>

    <div class="mt-6 text-center text-xs">
      <span class="crypto-tagline text-muted-foreground">
        {t(locale, 'loginCryptoTagline')}
      </span>
    </div>
  </div>
</div>

<style>
  .crypto-tagline {
    position: relative;
    display: inline-block;
    padding: 0.4rem 0.75rem;
    border-radius: 0.5rem;
    cursor: default;
    color: hsl(var(--muted-foreground));
    text-rendering: optimizeLegibility;
    transition:
      color 200ms ease,
      transform 200ms ease,
      filter 300ms ease;
  }
  .crypto-tagline::before {
    content: '';
    position: absolute;
    inset: -0.1rem -0.25rem;
    border-radius: 0.5rem;
    background: radial-gradient(
      60% 60% at 50% 60%,
      color-mix(in oklab, hsl(var(--primary)) 40%, transparent) 0%,
      color-mix(in oklab, hsl(var(--primary)) 12%, transparent) 60%,
      transparent 100%
    );
    filter: blur(12px);
    opacity: 0;
    pointer-events: none;
    transition: opacity 250ms ease;
  }
  .crypto-tagline::after {
    content: '';
    position: absolute;
    left: 50%;
    bottom: -2px;
    width: 0;
    height: 2px;
    background: linear-gradient(90deg, transparent, hsl(var(--primary)), transparent);
    opacity: 0.6;
    transform-origin: center;
    transition:
      width 260ms ease,
      left 260ms ease,
      opacity 260ms ease;
  }
  .crypto-tagline:hover {
    color: hsl(var(--foreground));
    transform: translateY(-1px);
    filter: drop-shadow(0 0 0.35rem color-mix(in oklab, hsl(var(--primary)) 45%, transparent));
  }
  .crypto-tagline:hover::before {
    opacity: 0.85;
  }
  .crypto-tagline:hover::after {
    width: 100%;
    left: 0;
    opacity: 1;
  }

  .blob-a {
    animation: blobPulseA 18s ease-in-out infinite both;
    transform-origin: center;
    will-change: transform, opacity;
    position: relative;
  }
  .blob-b {
    animation: blobPulseB 22s ease-in-out infinite both;
    animation-delay: 1.6s;
    transform-origin: center;
    will-change: transform, opacity;
    position: relative;
  }
  @keyframes blobPulseA {
    0% {
      transform: scale(0.96);
      opacity: 0.55;
    }
    50% {
      transform: scale(1.08);
      opacity: 0.9;
    }
    100% {
      transform: scale(0.96);
      opacity: 0.55;
    }
  }
  @keyframes blobPulseB {
    0% {
      transform: scale(1.04);
      opacity: 0.35;
    }
    50% {
      transform: scale(0.92);
      opacity: 0.7;
    }
    100% {
      transform: scale(1.04);
      opacity: 0.35;
    }
  }
</style>
