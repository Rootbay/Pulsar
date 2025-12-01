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
  import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Progress } from '$lib/components/ui/progress';
  import {
    isDatabaseLoaded,
    isLocked,
    needsPasswordSetup,
    totpVerified,
    totpRequired
  } from '$lib/stores';
  import { currentLocale } from '$lib/i18n';
  import { ArrowLeft } from '@lucide/svelte';

  let newMasterPassword = $state('');
  let confirmMasterPassword = $state('');
  let loginError = $state<string | null>(null);
  let isSetting = $state(false);
  let strengthScore = $state(0);

  type StrengthKey = 'Weak' | 'Medium' | 'Strong';
  let strengthLabel = $state<StrengthKey>('Weak');
  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  const locale = $derived($currentLocale as 'en' | 'sv');
  const strengthLabelText = $derived(
    strengthLabel === 'Strong'
      ? t(locale, 'Strong', 'Starkt')
      : strengthLabel === 'Medium'
        ? t(locale, 'Medium', 'Medel')
        : t(locale, 'Weak', 'Svagt')
  );

  const STRENGTH_META: Record<StrengthKey, { textClass: string; barClass: string }> = {
    Weak: {
      textClass: 'text-destructive',
      barClass: '[&_[data-slot=progress-indicator]]:bg-destructive'
    },
    Medium: {
      textClass: 'text-chart-warning',
      barClass: '[&_[data-slot=progress-indicator]]:bg-[color:var(--color-chart-5)]'
    },
    Strong: {
      textClass: 'text-chart-success',
      barClass: '[&_[data-slot=progress-indicator]]:bg-[color:var(--color-chart-4)]'
    }
  };

  let strengthMeta = $state(STRENGTH_META['Weak']);

  $effect(() => {
    strengthMeta = STRENGTH_META[strengthLabel];
  });

  $effect(() => {
    calculateStrength(newMasterPassword);
  });

  $effect(() => {
    if (!browser) {
      return;
    }

    if (!$isDatabaseLoaded) {
      goto('/select-vault', { replaceState: true });
    } else if (!$needsPasswordSetup) {
      goto($isLocked ? '/login' : '/', { replaceState: true });
    }
  });

  function calculateStrength(password: string) {
    let score = 0;
    if (password.length >= 12) score += 25;
    else if (password.length >= 8) score += 10;
    if (/[a-z]/.test(password)) score += 15;
    if (/[A-Z]/.test(password)) score += 15;
    if (/[0-9]/.test(password)) score += 15;
    if (/[^a-zA-Z0-9]/.test(password)) score += 20;
    if (password.length >= 16) score += 20;

    strengthScore = Math.min(100, score);

    if (strengthScore >= 75) {
      strengthLabel = 'Strong';
    } else if (strengthScore >= 40) {
      strengthLabel = 'Medium';
    } else {
      strengthLabel = 'Weak';
    }
  }

  async function handleSetMasterPassword() {
    if (!newMasterPassword.trim() || !confirmMasterPassword.trim()) {
      loginError = t(locale, 'Please fill in both password fields.', 'Fyll i båda lösenordsfälten.');
      return;
    }
    if (newMasterPassword !== confirmMasterPassword) {
      loginError = t(locale, 'Passwords do not match.', 'Lösenorden matchar inte.');
      return;
    }

    loginError = null;
    isSetting = true;
    try {
      await invoke('set_master_password', { password: newMasterPassword });
      needsPasswordSetup.set(false);
      isLocked.set(false);
      totpVerified.set(false);
      totpRequired.set(false);
    } catch (cause) {
      console.error('Set master password failed:', cause);
      loginError =
        typeof cause === 'string'
          ? cause
          : t(locale, 'An unknown error occurred.', 'Ett okänt fel inträffade.');
    } finally {
      isSetting = false;
    }
  }

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    void handleSetMasterPassword();
  }

  async function goBack() {
    try {
      await invoke('lock');
    } catch (error) {
      console.error('Failed to lock while leaving setup:', error);
    }
    isDatabaseLoaded.set(false);
    isLocked.set(true);
    needsPasswordSetup.set(false);
    totpVerified.set(false);
    totpRequired.set(false);
    goto('/select-vault', { replaceState: true });
  }
</script>

<div class="relative flex min-h-screen items-start justify-center bg-background px-4 pb-16 pt-20">
  <button
    type="button"
    class="absolute left-4 top-4 z-10 flex items-center gap-1 rounded-md px-2 py-1 text-sm text-muted-foreground transition hover:text-foreground"
    onclick={goBack}
  >
    <ArrowLeft class="h-4 w-4" />
    {t(locale, 'Back', 'Tillbaka')}
  </button>
  <div
    class="pointer-events-none absolute left-1/2 top-1/2 h-[min(90vw,32rem)] w-[min(90vw,32rem)] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl bg-primary-glow"
    aria-hidden="true"
  ></div>

  <Card class="relative z-10 w-full max-w-md border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <form class="flex flex-col gap-6" onsubmit={handleSubmit}>
      <CardHeader class="space-y-2 text-center">
        <CardTitle class="text-2xl font-semibold">
          {t(locale, 'Secure your vault', 'Säkra ditt valv')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Create a strong master password to protect every secret in Pulsar.',
            'Skapa ett starkt huvudlösenord för att skydda alla hemligheter i Pulsar.'
          )}
        </CardDescription>
      </CardHeader>

      <CardContent class="space-y-5">
        <div class="space-y-2">
          <Label for="new-password">
            {t(locale, 'New master password', 'Nytt huvudlösenord')}
          </Label>
          <Input
            id="new-password"
            type="password"
            placeholder={t(locale, 'New master password', 'Nytt huvudlösenord')}
            bind:value={newMasterPassword}
            autocomplete="new-password"
            disabled={isSetting}
          />
        </div>

        <div class="space-y-2">
          <Label for="confirm-password">
            {t(locale, 'Confirm master password', 'Bekräfta huvudlösenord')}
          </Label>
          <Input
            id="confirm-password"
            type="password"
            placeholder={t(locale, 'Confirm master password', 'Bekräfta huvudlösenord')}
            bind:value={confirmMasterPassword}
            autocomplete="new-password"
            disabled={isSetting}
          />
        </div>

        {#if newMasterPassword.length > 0}
          <div class="space-y-3 rounded-xl border border-border/60 bg-muted/20 p-4">
            <div class="flex items-center justify-between text-sm text-muted-foreground">
              <span class="font-medium">
                {t(locale, 'Strength', 'Styrka')}
              </span>
              <span class={`font-semibold ${strengthMeta.textClass}`}>{strengthLabelText}</span>
            </div>
            <Progress value={strengthScore} class={`h-2 ${strengthMeta.barClass}`} />
            <p class="text-xs text-muted-foreground">
              {t(
                locale,
                'Longer is better. Mix uppercase, lowercase, numbers, and symbols.',
                'Längre är bättre. Blanda stora och små bokstäver, siffror och symboler.'
              )}
            </p>
          </div>
        {/if}

        {#if loginError}
          <p class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm font-medium text-destructive">
            {loginError}
          </p>
        {/if}
      </CardContent>

      <CardFooter class="flex flex-col gap-2">
        <Button
          type="submit"
          class="w-full"
          disabled={
            isSetting ||
            !newMasterPassword.trim() ||
            newMasterPassword !== confirmMasterPassword
          }
        >
          {isSetting ? t(locale, 'Setting...', 'Sparar...') : t(locale, 'Set password', 'Sätt lösenord')}
        </Button>
        <p class="text-center text-xs text-muted-foreground">
          {t(
            locale,
            'This password stays on this device. Store it somewhere safe.',
            'Detta lösenord stannar på den här enheten. Förvara det på ett säkert ställe.'
          )}
        </p>
      </CardFooter>
    </form>
  </Card>
</div>
