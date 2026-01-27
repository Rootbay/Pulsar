<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { onDestroy, onMount } from 'svelte';
  import { callBackend } from '$lib/utils/backend';
  import { copyText } from '$lib/utils/copyHelper';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Spinner } from '$lib/components/ui/spinner/index.js';
  import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Copy, RefreshCw, ArrowLeft } from '@lucide/svelte';
  import { appState } from '$lib/stores';
  import { loginTotpSecret } from '$lib/stores/totp';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';

  const locale = $derived(i18n.locale);
  const t = (key: I18nKey, vars: Record<string, string | number> = {}) =>
    translate(locale, key, vars);

  const CODE_LENGTH = 6;
  const TOKEN_PERIOD = 30;

  let hiddenInput = $state<HTMLInputElement | null>(null);
  let code = $state('');
  let verificationError = $state<string | null>(null);
  let isVerifying = $state(false);

  let activeSecret = $state<string | null>(null);
  let currentToken = $state<string | null>(null);
  let tokenError = $state<string | null>(null);
  let isFetchingToken = $state(false);
  let timeRemaining = $state(0);

  type FeedbackVariant = 'success' | 'error';
  let copyFeedback = $state<{ message: string; variant: FeedbackVariant } | null>(null);
  let copyFeedbackTimeout: ReturnType<typeof setTimeout> | null = null;

  let countdownInterval: ReturnType<typeof setInterval> | null = null;
  let unsubscribeSecret: (() => void) | null = null;

  $effect(() => {
    if (
      browser &&
      !(
        appState.totpRequired &&
        !appState.totpVerified &&
        appState.isDatabaseLoaded &&
        !appState.isLocked &&
        !appState.needsPasswordSetup
      )
    ) {
      goto('/', { replaceState: true });
    }
  });

  const displayedToken = $derived(
    currentToken && currentToken.length
      ? currentToken.replace(/(.{3})/g, '$1 ').trim()
      : isFetchingToken && activeSecret
        ? '······'
        : '------'
  );

  const focusHiddenInput = () => {
    hiddenInput?.focus();
  };

  const sanitize = () => {
    const cleaned = code.replace(/\D/g, '').slice(0, CODE_LENGTH);
    if (cleaned !== code) {
      code = cleaned;
    }

    if (verificationError && code.length < CODE_LENGTH) {
      verificationError = null;
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      focusHiddenInput();
    }
  };

  const toErrorMessage = (error: unknown): string => {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    if (error && typeof error === 'object' && 'message' in error)
      return (error as { message: string }).message;
    return t('totpUnexpectedError');
  };

  function updateTimeRemaining() {
    if (!activeSecret) {
      timeRemaining = 0;
      return;
    }

    const epochSeconds = Math.floor(Date.now() / 1000);
    const remainder = epochSeconds % TOKEN_PERIOD;
    const remaining = TOKEN_PERIOD - remainder;
    timeRemaining = remaining === 0 ? TOKEN_PERIOD : remaining;
  }

  async function fetchToken(force = false) {
    if (!activeSecret) {
      currentToken = null;
      return;
    }

    if (isFetchingToken && !force) {
      return;
    }

    isFetchingToken = true;
    tokenError = null;

    try {
      const token = await callBackend<string>('generate_totp', { secret_b32: activeSecret });
      currentToken = token;
      updateTimeRemaining();
    } catch (error) {
      tokenError = toErrorMessage(error);
      currentToken = null;
    } finally {
      isFetchingToken = false;
    }
  }

  function startCountdown() {
    stopCountdown();
    updateTimeRemaining();

    countdownInterval = setInterval(() => {
      updateTimeRemaining();
      if (activeSecret && timeRemaining <= 1) {
        void fetchToken(true);
      }
    }, 1000);
  }

  function stopCountdown() {
    if (countdownInterval) {
      clearInterval(countdownInterval);
      countdownInterval = null;
    }
  }

  function setCopyFeedback(message: string, variant: FeedbackVariant) {
    copyFeedback = { message, variant };
    if (copyFeedbackTimeout) {
      clearTimeout(copyFeedbackTimeout);
    }

    copyFeedbackTimeout = setTimeout(() => {
      copyFeedback = null;
    }, 2500);
  }

  async function copyToken() {
    if (!currentToken) return;

    try {
      await copyText(currentToken);
      setCopyFeedback(t('totpCopySuccess'), 'success');
    } catch (error) {
      setCopyFeedback(toErrorMessage(error), 'error');
    }
  }

  function handleRegenerateToken() {
    void fetchToken(true);
  }

  const handleContinue = async () => {
    if (isVerifying) return;

    if (code.length !== CODE_LENGTH) {
      verificationError = t('totpEnterCodeError');
      return;
    }

    isVerifying = true;
    verificationError = null;

    try {
      await callBackend('verify_login_totp', { token: code });
      appState.isLocked = false;
      appState.totpVerified = true;
      appState.totpRequired = false;
      await goto('/', { replaceState: true });
    } catch (cause) {
      const message = toErrorMessage(cause);
      if (message.toLowerCase().includes('invalid')) {
        verificationError = t('totpInvalidCodeError');
      } else {
        verificationError = message;
      }
    } finally {
      isVerifying = false;
    }
  };

  onMount(() => {
    unsubscribeSecret = loginTotpSecret.subscribe((value) => {
      activeSecret = value;
      if (!value) {
        currentToken = null;
        tokenError = t('totpSecretMissing');
      } else {
        tokenError = null;
        void fetchToken(true);
      }
    });

    if (browser) {
      startCountdown();
    }
  });

  onDestroy(() => {
    unsubscribeSecret?.();
    stopCountdown();
    if (copyFeedbackTimeout) {
      clearTimeout(copyFeedbackTimeout);
      copyFeedbackTimeout = null;
    }
  });

  async function goBack() {
    try {
      await callBackend('lock');
    } catch (error) {
      console.error('Failed to lock while leaving TOTP screen:', error);
    }
    appState.totpRequired = false;
    appState.totpVerified = false;
    appState.isLocked = true;
    goto('/login', { replaceState: true });
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
  <link
    href="https://fonts.googleapis.com/css2?family=Inter:wght@100..900&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<div class="bg-background relative flex min-h-screen items-start justify-center px-4 pt-20 pb-16">
  <button
    type="button"
    class="text-muted-foreground hover:text-foreground absolute top-4 left-4 flex items-center gap-1 text-sm"
    onclick={goBack}
  >
    <ArrowLeft class="h-4 w-4" />
    {t('back')}
  </button>
  <div
    class="bg-primary-glow pointer-events-none absolute top-1/2 left-1/2 h-[min(90vw,32rem)] w-[min(90vw,32rem)] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl"
    aria-hidden="true"
  ></div>

  <Card
    class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 relative z-10 w-full max-w-md backdrop-blur"
  >
    <CardHeader class="space-y-2 text-center">
      <CardTitle class="text-2xl font-semibold">{t('totpTitle')}</CardTitle>
      <CardDescription>{t('totpSubtitle')}</CardDescription>
    </CardHeader>

    <CardContent class="space-y-6">
      <div
        class="border-border/60 bg-muted/20 hover:border-border relative flex gap-3 rounded-2xl border p-4 shadow-sm transition"
        role="button"
        tabindex="0"
        aria-label={t('totpAriaEnterCode')}
        onclick={focusHiddenInput}
        onkeydown={handleKeyDown}
      >
        {#each Array.from({ length: 6 }) as _, i (i)}
          <div
            class="border-border/50 bg-background text-foreground flex h-16 w-14 items-center justify-center rounded-xl border text-2xl font-semibold tracking-widest shadow-sm sm:h-20 sm:w-16 sm:text-3xl"
            aria-label={t('totpDigitLabel', { index: String(i + 1) })}
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

      <section class="border-border/60 bg-muted/10 rounded-2xl border p-5">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p class="text-foreground text-sm font-semibold">{t('totpCodeLabel')}</p>
            <p class="text-muted-foreground text-sm">{t('totpCodeHint')}</p>
          </div>
          <Badge variant={activeSecret ? 'default' : 'secondary'}>
            {activeSecret ? t('totpSecretStored') : t('totpSecretUnavailable')}
          </Badge>
        </div>

        <div class="mt-4 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div class="text-foreground font-mono text-3xl tracking-[0.4rem] sm:text-4xl">
            {displayedToken}
          </div>
          <div class="text-muted-foreground flex items-center gap-2 text-sm">
            {#if isFetchingToken}
              <Spinner class="h-4 w-4" aria-hidden="true" />
            {/if}
            <span>
              {t('totpRefreshIn', {
                seconds: timeRemaining > 0 ? `${timeRemaining}s` : '—'
              })}
            </span>
          </div>
        </div>

        {#if tokenError}
          <p class="text-destructive mt-3 text-sm" aria-live="polite">{tokenError}</p>
        {/if}

        <div class="mt-4 flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            class="gap-2"
            onclick={copyToken}
            disabled={!currentToken}
          >
            <Copy class="h-4 w-4" aria-hidden="true" />
            {t('totpCopyCode')}
          </Button>
          <Button
            type="button"
            variant="ghost"
            class="gap-2"
            onclick={handleRegenerateToken}
            disabled={!activeSecret || isFetchingToken}
          >
            <RefreshCw
              class={`h-4 w-4 ${isFetchingToken ? 'animate-spin' : ''}`}
              aria-hidden="true"
            />
            {t('totpRefresh')}
          </Button>
        </div>

        {#if copyFeedback}
          <p
            class={`mt-2 text-xs ${copyFeedback.variant === 'success' ? 'text-primary' : 'text-destructive'}`}
            aria-live="polite"
          >
            {copyFeedback.message}
          </p>
        {/if}

        <p class="text-muted-foreground mt-3 text-xs">
          {t('totpBackupNote')}
        </p>
      </section>
    </CardContent>

    <CardFooter class="flex flex-col gap-3">
      {#if verificationError}
        <p class="text-destructive text-sm font-medium" aria-live="assertive">
          {verificationError}
        </p>
      {/if}
      <Button
        type="button"
        class="w-full"
        onclick={handleContinue}
        disabled={isVerifying || code.length !== CODE_LENGTH}
      >
        {isVerifying ? t('totpVerifying') : t('totpVerifyButton')}
      </Button>
    </CardFooter>
  </Card>
</div>
