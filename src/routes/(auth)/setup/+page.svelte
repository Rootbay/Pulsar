<script lang="ts">
  import { browser } from '$app/environment';
  import { goto } from '$app/navigation';
  import { callBackend } from '$lib/utils/backend';
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
  import { appState } from '$lib/stores';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { ArrowLeft } from '@lucide/svelte';

  import { SecurityService } from '$lib/utils/security';

  const locale = $derived(i18n.locale);
  const t = (key: I18nKey, vars: Record<string, string | number> = {}) =>
    translate(locale, key, vars);

  let newMasterPassword = $state('');
  let confirmMasterPassword = $state('');
  let loginError = $state<string | null>(null);
  let isSetting = $state(false);

  let strengthResult = $derived(SecurityService.checkStrength(newMasterPassword));
  let strengthScore = $derived(newMasterPassword ? (strengthResult.score + 1) * 20 : 0);

  const STRENGTH_LABEL_KEYS = [
    'setupStrengthVeryWeak',
    'setupStrengthWeak',
    'setupStrengthFair',
    'setupStrengthStrong',
    'setupStrengthVeryStrong'
  ] as const;
  const STRENGTH_COLORS = [
    'text-destructive',
    'text-orange-500',
    'text-yellow-500',
    'text-emerald-500',
    'text-chart-success'
  ];
  const BAR_COLORS = [
    '[&_[data-slot=progress-indicator]]:bg-destructive',
    '[&_[data-slot=progress-indicator]]:bg-orange-500',
    '[&_[data-slot=progress-indicator]]:bg-yellow-500',
    '[&_[data-slot=progress-indicator]]:bg-emerald-500',
    '[&_[data-slot=progress-indicator]]:bg-[color:var(--color-chart-4)]'
  ];

  const strengthLabelText = $derived(
    newMasterPassword
      ? t(STRENGTH_LABEL_KEYS[strengthResult.score])
      : t('setupStrengthNone')
  );
  const strengthColorClass = $derived(
    newMasterPassword ? STRENGTH_COLORS[strengthResult.score] : 'text-muted-foreground'
  );
  const strengthBarClass = $derived(
    newMasterPassword ? BAR_COLORS[strengthResult.score] : 'bg-muted'
  );

  $effect(() => {
    if (!browser) {
      return;
    }

    if (!appState.isDatabaseLoaded) {
      goto('/select-vault', { replaceState: true });
    } else if (!appState.needsPasswordSetup) {
      goto(appState.isLocked ? '/login' : '/', { replaceState: true });
    }
  });

  async function handleSetMasterPassword() {
    if (!newMasterPassword.trim() || !confirmMasterPassword.trim()) {
      loginError = t('setupPasswordFieldsRequired');
      return;
    }
    if (newMasterPassword !== confirmMasterPassword) {
      loginError = t('setupPasswordMismatch');
      return;
    }

    loginError = null;
    isSetting = true;
    try {
      await callBackend('set_master_password', { password: newMasterPassword });
      appState.needsPasswordSetup = false;
      appState.isLocked = false;
      appState.totpVerified = false;
      appState.totpRequired = false;
    } catch (cause) {
      console.error('Set master password failed:', cause);
      loginError = typeof cause === 'string' ? cause : t('setupUnknownError');
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
      await callBackend('lock');
    } catch (error) {
      console.error('Failed to lock while leaving setup:', error);
    }
    appState.isDatabaseLoaded = false;
    appState.isLocked = true;
    appState.needsPasswordSetup = false;
    appState.totpVerified = false;
    appState.totpRequired = false;
    goto('/select-vault', { replaceState: true });
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
    class="text-muted-foreground hover:text-foreground absolute top-4 left-4 z-10 flex items-center gap-1 rounded-md px-2 py-1 text-sm transition"
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
    <form class="flex flex-col gap-6" onsubmit={handleSubmit}>
      <CardHeader class="space-y-2 text-center">
        <CardTitle class="text-2xl font-semibold">
          {t('setupTitle')}
        </CardTitle>
        <CardDescription>
          {t('setupSubtitle')}
        </CardDescription>
      </CardHeader>

      <CardContent class="space-y-5">
        <div class="space-y-2">
          <Label for="new-password">
            {t('setupNewMasterPassword')}
          </Label>
          <Input
            id="new-password"
            type="password"
            placeholder={t('setupNewMasterPassword')}
            bind:value={newMasterPassword}
            autocomplete="new-password"
            disabled={isSetting}
          />
        </div>

        <div class="space-y-2">
          <Label for="confirm-password">
            {t('setupConfirmMasterPassword')}
          </Label>
          <Input
            id="confirm-password"
            type="password"
            placeholder={t('setupConfirmMasterPassword')}
            bind:value={confirmMasterPassword}
            autocomplete="new-password"
            disabled={isSetting}
          />
        </div>

        {#if newMasterPassword.length > 0}
          <div class="border-border/60 bg-muted/20 space-y-3 rounded-xl border p-4">
            <div class="text-muted-foreground flex items-center justify-between text-sm">
              <span class="font-medium">
                {t('setupStrength')}
              </span>
              <span class={`font-semibold ${strengthColorClass}`}>{strengthLabelText}</span>
            </div>
            <Progress value={strengthScore} class={`h-2 ${strengthBarClass}`} />
            {#if strengthResult.feedback.warning || strengthResult.feedback.suggestions.length > 0}
              <p class="text-muted-foreground text-xs leading-relaxed">
                {strengthResult.feedback.warning ? strengthResult.feedback.warning + '. ' : ''}
                {strengthResult.feedback.suggestions[0] || ''}
              </p>
            {:else}
              <p class="text-muted-foreground text-xs">
                {t('setupPasswordTip')}
              </p>
            {/if}
          </div>
        {/if}

        {#if confirmMasterPassword && newMasterPassword !== confirmMasterPassword}
          <p class="text-destructive text-xs font-medium">
            {t('setupPasswordMismatch')}
          </p>
        {/if}

        {#if loginError}
          <p
            class="border-destructive/30 bg-destructive/10 text-destructive rounded-md border px-3 py-2 text-sm font-medium"
          >
            {loginError}
          </p>
        {/if}
      </CardContent>

      <CardFooter class="flex flex-col gap-2">
        <Button
          type="submit"
          class="w-full"
          disabled={isSetting ||
            !newMasterPassword.trim() ||
            newMasterPassword !== confirmMasterPassword}
        >
          {isSetting ? t('setupSetting') : t('setupSetPassword')}
        </Button>
        <p class="text-muted-foreground text-center text-xs">
          {t('setupPasswordLocalNote')}
        </p>
      </CardFooter>
    </form>
  </Card>
</div>
