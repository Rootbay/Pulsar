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
    totpVerified
  } from '$lib/stores';

  let newMasterPassword = $state('');
  let confirmMasterPassword = $state('');
  let loginError = $state<string | null>(null);
  let isSetting = $state(false);
  let strengthScore = $state(0);

  type StrengthKey = 'Weak' | 'Medium' | 'Strong';
  let strengthLabel = $state<StrengthKey>('Weak');

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

  $: if (browser) {
    if (!$isDatabaseLoaded) {
      goto('/select-vault', { replaceState: true });
    } else if (!$needsPasswordSetup) {
      goto($isLocked ? '/login' : '/', { replaceState: true });
    }
  }

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
      loginError = 'Please fill in both password fields.';
      return;
    }
    if (newMasterPassword !== confirmMasterPassword) {
      loginError = 'Passwords do not match.';
      return;
    }

    loginError = null;
    isSetting = true;
    try {
      await invoke('set_master_password', { password: newMasterPassword });
      needsPasswordSetup.set(false);
      isLocked.set(false);
      totpVerified.set(false);
    } catch (cause) {
      console.error('Set master password failed:', cause);
      loginError = typeof cause === 'string' ? cause : 'An unknown error occurred.';
    } finally {
      isSetting = false;
    }
  }

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    void handleSetMasterPassword();
  }
</script>

<div class="relative flex min-h-screen items-start justify-center bg-background px-4 pb-16 pt-20">
  <div
    class="pointer-events-none absolute left-1/2 top-1/2 h-[min(90vw,32rem)] w-[min(90vw,32rem)] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl bg-primary-glow"
    aria-hidden="true"
  ></div>

  <Card class="relative z-10 w-full max-w-md border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <form class="flex flex-col gap-6" onsubmit={handleSubmit}>
      <CardHeader class="space-y-2 text-center">
        <CardTitle class="text-2xl font-semibold">Secure your vault</CardTitle>
        <CardDescription>Create a strong master password to protect every secret in Pulsar.</CardDescription>
      </CardHeader>

      <CardContent class="space-y-5">
        <div class="space-y-2">
          <Label for="new-password">New master password</Label>
          <Input
            id="new-password"
            type="password"
            placeholder="New master password"
            bind:value={newMasterPassword}
            autocomplete="new-password"
            disabled={isSetting}
          />
        </div>

        <div class="space-y-2">
          <Label for="confirm-password">Confirm master password</Label>
          <Input
            id="confirm-password"
            type="password"
            placeholder="Confirm master password"
            bind:value={confirmMasterPassword}
            autocomplete="new-password"
            disabled={isSetting}
          />
        </div>

        {#if newMasterPassword.length > 0}
          <div class="space-y-3 rounded-xl border border-border/60 bg-muted/20 p-4">
            <div class="flex items-center justify-between text-sm text-muted-foreground">
              <span class="font-medium">Strength</span>
              <span class={`font-semibold ${strengthMeta.textClass}`}>{strengthLabel}</span>
            </div>
            <Progress value={strengthScore} class={`h-2 ${strengthMeta.barClass}`} />
            <p class="text-xs text-muted-foreground">
              Longer is better. Mix uppercase, lowercase, numbers, and symbols.
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
          {isSetting ? 'Setting...' : 'Set password'}
        </Button>
        <p class="text-center text-xs text-muted-foreground">
          This password stays on this device. Store it somewhere safe.
        </p>
      </CardFooter>
    </form>
  </Card>
</div>
