<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link
    href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { isDatabaseLoaded, isLocked, needsPasswordSetup, totpVerified, totpRequired } from '$lib/stores';
  import { currentLocale } from '$lib/i18n';
  import { Lock, Loader2, Eye, EyeOff, ArrowLeft } from '@lucide/svelte';

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  $: locale = $currentLocale as 'en' | 'sv';

  let password = '';
  let loginError: string | null = null;
  let isUnlocking = false;
  let showPassword = false;

  $: if (browser) {
    if (!$isDatabaseLoaded) {
      goto('/select-vault', { replaceState: true });
    } else if ($needsPasswordSetup) {
      goto('/setup', { replaceState: true });
    } else if (!$isLocked) {
      goto('/', { replaceState: true });
    }
  }

  const handleUnlock = async () => {
    const trimmedPassword = password.trim();
    if (isUnlocking || !trimmedPassword) return;

    isUnlocking = true;
    loginError = null;

    try {
      const result = await invoke<{ totp_required: boolean }>('unlock', { password: trimmedPassword });
      if (result?.totp_required) {
        totpRequired.set(true);
        totpVerified.set(false);
        isLocked.set(false);
        await goto('/totp', { replaceState: true });
      } else {
        totpRequired.set(false);
        totpVerified.set(true);
        isLocked.set(false);
        await goto('/', { replaceState: true });
      }
    } catch (error) {
      console.error('Unlock failed:', error);
      totpRequired.set(false);
      totpVerified.set(false);
      loginError =
        typeof error === 'string'
          ? error
          : t(locale, 'An unknown error occurred.', 'Ett okänt fel inträffade.');
    } finally {
      isUnlocking = false;
    }
  };

  const handleChangeDatabase = async () => {
    await invoke('lock');
    isDatabaseLoaded.set(false);
    isLocked.set(true);
    totpRequired.set(false);
    totpVerified.set(false);
  };

  const goBack = async () => {
    await handleChangeDatabase();
    await goto('/select-vault', { replaceState: true });
  };

  const handleSubmit = (event: SubmitEvent) => {
    event.preventDefault();
    void handleUnlock();
  };

  $: canSubmit = password.trim().length > 0 && !isUnlocking;

  // Background blobs: keep only slow pulse (no mouse tracking)
</script>

<!-- Background -->
<div class="relative min-h-screen bg-gradient-to-b from-background to-background/80">
    <button
      type="button"
      class="absolute left-4 top-4 z-10 flex items-center gap-1 rounded-md px-2 py-1 text-sm text-muted-foreground transition hover:text-foreground"
      onclick={goBack}
    >
      <ArrowLeft class="h-4 w-4" />
      {t(locale, 'Back', 'Tillbaka')}
    </button>
    <div class="pointer-events-none absolute inset-0 -z-10">
      <div class="absolute left-[10%] top-[10%] size-[28rem] rounded-full bg-primary/10 blur-3xl blob-a"></div>
      <div class="absolute right-[10%] bottom-[12%] size-[22rem] rounded-full bg-muted/40 blur-2xl blob-b"></div>
    </div>

  <!-- Centered card -->
  <div class="mx-auto grid min-h-screen w-full place-items-center px-4">
    <Card class="w-full max-w-md border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
      <form class="flex flex-col" onsubmit={handleSubmit}>
        <CardHeader class="space-y-0 text-center">
          <div class="mx-auto mb-2 flex size-12 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Lock class="size-6" />
          </div>
          <CardTitle class="text-2xl font-semibold tracking-tight">
            {t(locale, 'Welcome back', 'Välkommen tillbaka')}
          </CardTitle>
          <CardDescription class="mt-0">
            {t(locale, 'Unlock your vault with your master password', 'Lås upp ditt valv med ditt huvudlösenord')}
          </CardDescription>
        </CardHeader>

        <CardContent class="mt-6 space-y-4">
          <div class="space-y-2">
            <Label for="master-password">
              {t(locale, 'Master password', 'Huvudlösenord')}
            </Label>
            <div class="relative">
              <Input
                id="master-password"
                type={showPassword ? 'text' : 'password'}
                placeholder={t(locale, 'Enter your master password', 'Ange ditt huvudlösenord')}
                bind:value={password}
                autocomplete="current-password"
                disabled={isUnlocking}
                class="pr-10"
              />
              <button
                type="button"
                class="absolute inset-y-0 right-0 flex items-center px-3 text-muted-foreground hover:text-foreground"
                onclick={() => (showPassword = !showPassword)}
                aria-label={
                  showPassword
                    ? t(locale, 'Hide password', 'Dölj lösenord')
                    : t(locale, 'Show password', 'Visa lösenord')
                }
                tabindex="-1"
              >
                {#if showPassword}
                  <EyeOff class="size-4" />
                {:else}
                  <Eye class="size-4" />
                {/if}
              </button>
            </div>
          </div>

          {#if loginError}
            <p class="text-sm font-medium text-destructive">{loginError}</p>
          {/if}
        </CardContent>

        <CardFooter class="mt-6 flex flex-col gap-2">
          <Button type="submit" class="w-full" disabled={!canSubmit}>
            {#if isUnlocking}
              <Loader2 class="mr-2 size-4 animate-spin" /> {t(locale, 'Unlocking…', 'Låser upp…')}
            {:else}
              {t(locale, 'Unlock', 'Lås upp')}
            {/if}
          </Button>
          <Button type="button" variant="ghost" class="w-full" onclick={handleChangeDatabase}>
            {t(locale, 'Open another vault', 'Öppna ett annat valv')}
          </Button>
        </CardFooter>
      </form>
    </Card>

    <div class="mt-6 text-center text-xs">
      <span class="crypto-tagline text-muted-foreground">
        {t(locale, 'Secure by Argon2id + XChaCha20-Poly1305', 'Säker med Argon2id + XChaCha20-Poly1305')}
      </span>
    </div>
  </div>
</div>

<style>
  .crypto-tagline {
    position: relative;
    display: inline-block;
    padding: 0.4rem 0.75rem; /* larger hover hit-area */
    border-radius: 0.5rem;
    cursor: default;
    color: hsl(var(--muted-foreground));
    text-rendering: optimizeLegibility;
    transition: color 200ms ease, transform 200ms ease, filter 300ms ease;
  }
  /* subtle glow container */
  .crypto-tagline::before {
    content: '';
    position: absolute;
    inset: -0.1rem -0.25rem; /* extend glow slightly beyond padded area */
    border-radius: .5rem;
    background: radial-gradient(60% 60% at 50% 60%,
      color-mix(in oklab, hsl(var(--primary)) 40%, transparent) 0%,
      color-mix(in oklab, hsl(var(--primary)) 12%, transparent) 60%,
      transparent 100%);
    filter: blur(12px);
    opacity: 0;
    pointer-events: none;
    transition: opacity 250ms ease;
  }
  /* animated underline */
  .crypto-tagline::after {
    content: '';
    position: absolute;
    left: 50%;
    bottom: -2px;
    width: 0;
    height: 2px;
    background: linear-gradient(90deg, transparent, hsl(var(--primary)), transparent);
    opacity: .6;
    transform-origin: center;
    transition: width 260ms ease, left 260ms ease, opacity 260ms ease;
  }
  .crypto-tagline:hover {
    color: hsl(var(--foreground));
    transform: translateY(-1px);
    filter: drop-shadow(0 0 0.35rem color-mix(in oklab, hsl(var(--primary)) 45%, transparent));
  }
  .crypto-tagline:hover::before { opacity: .85; }
  .crypto-tagline:hover::after { width: 100%; left: 0; opacity: 1; }

  /* slow, subtle background pulses */
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
    0% { transform: scale(0.96); opacity: 0.55; }
    50% { transform: scale(1.08); opacity: 0.9; }
    100% { transform: scale(0.96); opacity: 0.55; }
  }
  @keyframes blobPulseB {
    0% { transform: scale(1.04); opacity: 0.35; }
    50% { transform: scale(0.92); opacity: 0.7; }
    100% { transform: scale(1.04); opacity: 0.35; }
  }

  /* (no mouse-responsive overlay; keep gentle pulses only) */
</style>







