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
  import {
    isDatabaseLoaded,
    isLocked,
    needsPasswordSetup,
    totpVerified
  } from '$lib/stores';
  import { securitySettings } from '$lib/stores/security';

  let hiddenInput: HTMLInputElement | null = null;
  let code = '';

  $: {
    if (
      browser &&
      (!($securitySettings.useTotp && !$totpVerified && $isDatabaseLoaded && !$isLocked && !$needsPasswordSetup))
    ) {
      goto('/', { replaceState: true });
    }
  }

  const focusHiddenInput = () => {
    hiddenInput?.focus();
  };

  const sanitize = () => {
    code = code.replace(/\D/g, '').slice(0, 6);
  };

  const handleContinue = async () => {
    if (code.length !== 6) {
      return;
    }

    try {
      const isValid = await invoke<boolean>('verify_totp', { token: code });
      if (isValid) {
        totpVerified.set(true);
        await goto('/', { replaceState: true });
      }
    } catch (cause) {
      console.error('Failed to verify TOTP:', cause);
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      focusHiddenInput();
    }
  };
</script>

<div class="relative flex min-h-screen items-start justify-center bg-background px-4 pb-16 pt-20">
  <div
    class="pointer-events-none absolute left-1/2 top-1/2 h-[min(90vw,32rem)] w-[min(90vw,32rem)] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl bg-primary-glow"
    aria-hidden="true"
  ></div>

  <Card class="relative z-10 w-full max-w-md border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="space-y-2 text-center">
      <CardTitle class="text-2xl font-semibold">Unlock Pulsar Pass</CardTitle>
      <CardDescription>Verification step with a TOTP-generated key</CardDescription>
    </CardHeader>

    <CardContent class="flex flex-col gap-6">
      <div
        class="relative flex gap-3 rounded-2xl border border-border/60 bg-muted/20 p-4 shadow-sm transition hover:border-border"
        role="button"
        tabindex="0"
        aria-label="Enter TOTP code"
        onclick={focusHiddenInput}
        onkeydown={handleKeyDown}
      >
        {#each Array.from({ length: 6 }) as _, i (i)}
          <div
            class="flex h-16 w-14 items-center justify-center rounded-xl border border-border/50 bg-background text-2xl font-semibold tracking-widest text-foreground shadow-sm sm:h-20 sm:w-16 sm:text-3xl"
            aria-label={`Digit ${i + 1}`}
          >
            {code[i] ?? ''}
          </div>
        {/each}

        <input
          type="text"
          bind:this={hiddenInput}
          bind:value={code}
          maxlength="6"
          inputmode="numeric"
          pattern="[0-9]*"
          autocomplete="one-time-code"
          oninput={sanitize}
          class="absolute inset-0 h-full w-full cursor-text opacity-0"
        />
      </div>
    </CardContent>

    <CardFooter class="flex flex-col gap-2">
      <Button type="button" class="w-full" onclick={handleContinue} disabled={code.length !== 6}>
        Continue
      </Button>
      <Button type="button" class="w-full" variant="outline" disabled>
        Generate
      </Button>
    </CardFooter>
  </Card>
</div>
