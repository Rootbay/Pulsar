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
  import {
    isDatabaseLoaded,
    isLocked,
    needsPasswordSetup,
    totpVerified,
    totpRequired
  } from '$lib/stores';

  let password = '';
  let loginError: string | null = null;
  let isUnlocking = false;

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
    if (isUnlocking || !trimmedPassword) {
      return;
    }

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
      loginError = typeof error === 'string' ? error : 'An unknown error occurred.';
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

  const handleSubmit = (event: SubmitEvent) => {
    event.preventDefault();
    void handleUnlock();
  };
</script>

<div class="relative flex min-h-screen items-start justify-center bg-background px-4 pb-16 pt-20">
  <div
    class="pointer-events-none absolute left-1/2 top-1/2 h-[min(90vw,32rem)] w-[min(90vw,32rem)] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl bg-primary-glow"
    aria-hidden="true"
  ></div>

  <Card class="relative z-10 w-full max-w-md border-border/60 bg-card/80 p-0 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <form class="flex flex-col gap-6" onsubmit={handleSubmit}>
      <CardHeader class="space-y-2 text-center">
        <CardTitle class="text-2xl font-semibold">Unlock Pulsar Pass</CardTitle>
        <CardDescription>Enter your master password to unlock your vault.</CardDescription>
      </CardHeader>

      <CardContent class="space-y-4">
        <div class="space-y-2">
          <Label for="master-password">Master password</Label>
          <Input
            id="master-password"
            type="password"
            placeholder="Master password"
            bind:value={password}
            autocomplete="current-password"
            disabled={isUnlocking}
          />
        </div>

        {#if loginError}
          <p class="text-sm font-medium text-destructive">{loginError}</p>
        {/if}
      </CardContent>

      <CardFooter class="flex flex-col gap-2">
        <Button type="submit" class="w-full" disabled={isUnlocking || !password.trim()}>
          {isUnlocking ? 'Unlocking...' : 'Continue'}
        </Button>
        <Button type="button" variant="ghost" class="w-full" onclick={handleChangeDatabase}>
          Change Database
        </Button>
      </CardFooter>
    </form>
  </Card>
</div>
