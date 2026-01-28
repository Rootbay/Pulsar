<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { appState } from '$lib/stores';
  import { settings } from '$lib/stores/appSettings.svelte';
  import { loginTotpStore } from '$lib/stores/totp.svelte';
  import { callBackend } from '$lib/utils/backend';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from '$lib/components/ui/card';
  import { Label } from '$lib/components/ui/label';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import Input from '$lib/components/ui/FieldInput.svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertTitle, AlertDescription } from '$lib/components/ui/alert';
  import { Spinner } from '$lib/components/ui/spinner/index.js';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle
  } from '$lib/components/ui/dialog';
  import {
    ShieldCheck,
    Lock,
    KeyRound,
    Trash2,
    RefreshCw,
    FingerprintPattern,
    TriangleAlert,
    Eye,
    EyeOff,
    Copy,
    CalendarClock,
    CircleAlert,
    Check,
    QrCode,
    Link2,
    HardDrive
  } from '@lucide/svelte';
  import { i18n, t as translate, type I18nKey } from '$lib/i18n.svelte';
  import { parseError } from '$lib/utils/error';
  import { SecurityService } from '$lib/utils/security';
  import type { SecuritySettings } from '$lib/config/settings';
  import { cn } from '$lib/utils';
  import { toast } from '$lib/components/ui/sonner';
  import { copyText } from '$lib/utils/copyHelper';
  import { clipboardService } from '$lib/utils/clipboardService.svelte';

  import { securityDashboard } from '$lib/stores/security-dashboard.svelte';

  interface Argon2Params {
    memoryKib: number;
    timeCost: number;
    parallelism: number;
  }

  type SecurityActionId = 'rekey' | 'wipe-memory' | 'integrity-check';

  interface CopyFeedback {
    context: 'pending' | 'stored';
    message: string;
    variant: 'success' | 'error';
  }

  const TOTP_ISSUER = 'Pulsar';
  const TOTP_ACCOUNT = 'User';
  const TOTP_CODE_LENGTH = 6;
  const MIN_PASSWORD_LENGTH = 8;

  const locale = $derived(i18n.locale);
  const t = (key: string, vars = {}) => translate(locale, key as I18nKey, vars);

  let currentSettings = $derived(settings.state.security);
  let currentClipboardSettings = $derived(settings.state.clipboard);

  const currentProvisioningUri = $derived(
    loginTotpStore.secret ? buildProvisioningUri(loginTotpStore.secret) : null
  );
  const generateButtonLabel = $derived(
    loginTotpStore.secret ? t('Regenerate Secret') : t('Generate Secret')
  );

  let passwordModalOpen = $state(false);
  let showCurrentPassword = $state(false);
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let changePasswordError = $state('');
  let isChangingPassword = $state(false);

  let kdfModalOpen = $state(false);
  let kdfCurrentPassword = $state('');
  let showKdfPassword = $state(false);
  let kdfMemoryMb = $state('64');
  let kdfTimeCost = $state('3');
  let kdfParallelism = $state('4');
  let kdfError = $state('');
  let isUpdatingKdf = $state(false);

  let argon2Params = $state<Argon2Params>({
    memoryKib: 64 * 1024,
    timeCost: 3,
    parallelism: 4
  });
  let argon2Loading = $state(false);

  let securityActionPending = $state<Record<SecurityActionId, boolean>>({
    rekey: false,
    'wipe-memory': false,
    'integrity-check': false
  });

  let healthReport = $derived(securityDashboard.lastReport);
  let problematicItems = $derived(securityDashboard.problematicItems);
  let healthLoading = $state(false);
  let healthError = $state<string | null>(null);

  let isBiometricsEnabled = $state(false);
  let isBiometricActionLoading = $state(false);
  let biometricModalOpen = $state(false);
  let biometricPassword = $state('');
  let showBiometricPassword = $state(false);
  let biometricError = $state('');

  let pendingTotpSecret = $state<string | null>(null);
  let pendingProvisioningUri = $state<string | null>(null);
  let totpVerificationCode = $state('');
  let totpVerificationError = $state<string | null>(null);
  let totpGenerationError = $state<string | null>(null);
  let secretCopyFeedback = $state<CopyFeedback | null>(null);
  let secretCopyTimeout: ReturnType<typeof setTimeout> | null = null;
  let uriCopyFeedback = $state<CopyFeedback | null>(null);
  let uriCopyTimeout: ReturnType<typeof setTimeout> | null = null;
  let isTotpStatusLoading = $state(false);
  let isGeneratingTotpSecret = $state(false);
  let isConfirmingTotp = $state(false);
  let isDisablingTotp = $state(false);
  let totpStatusError = $state<string | null>(null);
  let totpSetupSuccess = $state<string | null>(null);
  let totpDisableSuccess = $state<string | null>(null);

  const memoryFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
  const gigabyteFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

  const toErrorMessage = parseError;

  const formatSecret = (secret: string) => secret.replace(/(.{4})/g, '$1 ').trim();

  async function loadSecurityReport() {
    healthLoading = true;
    healthError = null;
    try {
      await securityDashboard.runAudit();
    } catch (error) {
      healthError = parseError(error);
    } finally {
      healthLoading = false;
    }
  }

  function navigateToItem(id: number) {
    appState.requestedItemId = id;
    appState.showSettingsPopup = false;
  }

  function getProblematicItem(id: number) {
    return problematicItems.find((i) => i.id === id);
  }

  async function loadBiometricsStatus() {
    try {
      isBiometricsEnabled = await callBackend<boolean>('is_biometrics_enabled');
    } catch (error) {
      console.error('Failed to check biometric status:', error);
    }
  }

  async function handleBiometricToggle() {
    if (isBiometricActionLoading) return;

    if (isBiometricsEnabled) {
      isBiometricActionLoading = true;
      try {
        await callBackend('disable_biometrics');
        isBiometricsEnabled = false;
        toast.success('Biometric unlock disabled.');
      } catch (error) {
        toast.error(`Failed to disable biometrics: ${parseError(error)}`);
      } finally {
        isBiometricActionLoading = false;
      }
    } else {
      biometricModalOpen = true;
      biometricPassword = '';
      biometricError = '';
    }
  }

  async function submitBiometricEnable() {
    if (!biometricPassword) {
      biometricError = 'Password is required.';
      return;
    }

    isBiometricActionLoading = true;
    biometricError = '';

    try {
      const { authenticate } = await import('@tauri-apps/plugin-biometric');
      await authenticate('Verify identity to enable biometric unlock');

      await callBackend('enable_biometrics', { password: biometricPassword });
      isBiometricsEnabled = true;
      biometricModalOpen = false;
      toast.success('Biometric unlock enabled.');
    } catch (error) {
      biometricError = parseError(error);
    } finally {
      isBiometricActionLoading = false;
    }
  }

  function toggleBiometricPasswordVisibility() {
    showBiometricPassword = !showBiometricPassword;
  }

  function handleBiometricDialogChange(open: boolean) {
    biometricModalOpen = open;
    if (!open) {
      biometricPassword = '';
      biometricError = '';
      isBiometricActionLoading = false;
    }
  }

  function buildProvisioningUri(secret: string) {
    const label = encodeURIComponent(`${TOTP_ISSUER}:${TOTP_ACCOUNT}`);
    const issuer = encodeURIComponent(TOTP_ISSUER);
    return `otpauth://totp/${label}?secret=${secret}&issuer=${issuer}`;
  }

  function setSecretCopyFeedback(
    context: CopyFeedback['context'],
    message: string,
    variant: CopyFeedback['variant']
  ) {
    secretCopyFeedback = { context, message, variant };
    if (secretCopyTimeout) {
      clearTimeout(secretCopyTimeout);
    }

    secretCopyTimeout = setTimeout(() => {
      secretCopyFeedback = null;
    }, 2500);
  }

  function setUriCopyFeedback(
    context: CopyFeedback['context'],
    message: string,
    variant: CopyFeedback['variant']
  ) {
    uriCopyFeedback = { context, message, variant };
    if (uriCopyTimeout) {
      clearTimeout(uriCopyTimeout);
    }

    uriCopyTimeout = setTimeout(() => {
      uriCopyFeedback = null;
    }, 2500);
  }

  function clearCopyTimeouts() {
    if (secretCopyTimeout) {
      clearTimeout(secretCopyTimeout);
      secretCopyTimeout = null;
    }
    if (uriCopyTimeout) {
      clearTimeout(uriCopyTimeout);
      uriCopyTimeout = null;
    }
  }

  function sanitizeTotpCode() {
    const cleaned = totpVerificationCode.replace(/\D/g, '').slice(0, TOTP_CODE_LENGTH);
    if (cleaned !== totpVerificationCode) {
      totpVerificationCode = cleaned;
    }

    if (totpVerificationError && totpVerificationCode.length < TOTP_CODE_LENGTH) {
      totpVerificationError = null;
    }
  }

  function cancelTotpSetup() {
    pendingTotpSecret = null;
    pendingProvisioningUri = null;
    totpVerificationCode = '';
    totpVerificationError = null;
    totpGenerationError = null;

    if (secretCopyFeedback?.context === 'pending') {
      secretCopyFeedback = null;
    }
    if (uriCopyFeedback?.context === 'pending') {
      uriCopyFeedback = null;
    }
  }

  async function refreshTotpStatus() {
    isTotpStatusLoading = true;
    totpStatusError = null;

    try {
      const configured = await callBackend<boolean>('is_login_totp_configured');
      loginTotpStore.configured = configured;
      if (configured) {
        const secret = await callBackend<string | null>('get_login_totp_secret');
        loginTotpStore.secret = secret;
      } else {
        loginTotpStore.secret = null;
      }
    } catch (error) {
      totpStatusError = parseError(error);
    } finally {
      isTotpStatusLoading = false;
    }
  }

  async function handleGenerateTotpSecret() {
    if (isGeneratingTotpSecret) return;

    totpGenerationError = null;
    totpStatusError = null;
    totpSetupSuccess = null;
    totpDisableSuccess = null;

    isGeneratingTotpSecret = true;

    try {
      const secret = await callBackend<string>('generate_totp_secret');
      pendingTotpSecret = secret;
      pendingProvisioningUri = buildProvisioningUri(secret);
      totpVerificationCode = '';
      totpVerificationError = null;

      if (secretCopyFeedback?.context === 'pending') {
        secretCopyFeedback = null;
      }
      if (uriCopyFeedback?.context === 'pending') {
        uriCopyFeedback = null;
      }
    } catch (error) {
      totpGenerationError = parseError(error);
    } finally {
      isGeneratingTotpSecret = false;
    }
  }

  async function copySecret(secret: string | null, context: CopyFeedback['context']) {
    if (!secret) return;

    try {
      await copyText(secret, 'TOTP Secret');
      setSecretCopyFeedback(context, 'Secret copied to clipboard.', 'success');
    } catch (error) {
      setSecretCopyFeedback(context, parseError(error), 'error');
    }
  }

  async function copyProvisioningUri(uri: string | null, context: CopyFeedback['context']) {
    if (!uri) return;

    try {
      await copyText(uri, 'TOTP Setup Link');
      setUriCopyFeedback(context, 'Setup link copied to clipboard.', 'success');
    } catch (error) {
      setUriCopyFeedback(context, parseError(error), 'error');
    }
  }

  async function handleConfirmTotp() {
    if (!pendingTotpSecret || isConfirmingTotp) {
      return;
    }

    if (totpVerificationCode.length !== TOTP_CODE_LENGTH) {
      totpVerificationError = 'Enter the 6-digit token from your authenticator.';
      return;
    }

    isConfirmingTotp = true;
    totpVerificationError = null;

    try {
      const isValid = await callBackend<boolean>('verify_totp_secret', {
        secret_b32: pendingTotpSecret,
        token: totpVerificationCode
      });
      if (!isValid) {
        totpVerificationError =
          'The verification code did not match. Wait for the next code window and try again.';
        return;
      }

      await callBackend('configure_login_totp', { secret_b32: pendingTotpSecret });
      loginTotpStore.secret = pendingTotpSecret;
      loginTotpStore.configured = true;
      totpSetupSuccess =
        'Login TOTP is now enabled. The new secret has been stored on this device.';
      totpDisableSuccess = null;
      pendingTotpSecret = null;
      pendingProvisioningUri = null;
      totpVerificationCode = '';
      appState.totpRequired = false;
      appState.totpVerified = true;
      await refreshTotpStatus();
    } catch (error) {
      totpVerificationError = parseError(error);
    } finally {
      isConfirmingTotp = false;
    }
  }

  async function handleDisableTotp() {
    if (isDisablingTotp) return;

    isDisablingTotp = true;
    totpStatusError = null;
    totpGenerationError = null;
    totpVerificationError = null;
    totpSetupSuccess = null;

    try {
      await callBackend('disable_login_totp');
      loginTotpStore.configured = false;
      loginTotpStore.secret = null;
      pendingTotpSecret = null;
      pendingProvisioningUri = null;
      totpVerificationCode = '';
      totpDisableSuccess = 'Login TOTP disabled. Unlocking will only require your master password.';
      appState.totpRequired = false;
      appState.totpVerified = false;
      await refreshTotpStatus();
    } catch (error) {
      totpStatusError = parseError(error);
    } finally {
      isDisablingTotp = false;
    }
  }

  onMount(() => {
    loadArgon2Params();
    loadBiometricsStatus();
    loadSecurityReport();
    refreshTotpStatus();
  });

  $effect(() => {
    if (appState.isLocked) {
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
      kdfCurrentPassword = '';
      biometricPassword = '';
      pendingTotpSecret = null;
      pendingProvisioningUri = null;
      totpVerificationCode = '';
      healthReport = null;

      passwordModalOpen = false;
      kdfModalOpen = false;
      biometricModalOpen = false;
    }
  });

  onDestroy(() => {
    clearCopyTimeouts();
  });

  function applyChanges(partial: Partial<SecuritySettings>) {
    const updated = { ...settings.state.security, ...partial };
    settings.state.security = updated;
    settings.save();
  }

  function toggleSetting(setting: keyof SecuritySettings) {
    const val = currentSettings[setting];
    if (typeof val === 'boolean') {
      applyChanges({ [setting]: !val } as Partial<SecuritySettings>);
    }
  }

  function updateAutoLock(value: SecuritySettings['autoLockInactivity']) {
    applyChanges({ autoLockInactivity: value });
  }

  function updateLockGrace(value: string) {
    const seconds = parseInt(value);
    if (!isNaN(seconds)) {
      applyChanges({ lockGraceSeconds: Math.min(Math.max(seconds, 0), 60) });
    }
  }

  function updateClipboardClear(value: string) {
    const seconds = parseInt(value);
    if (!isNaN(seconds)) {
      clipboardService.updateSettings({ clearAfterDuration: seconds });
    }
  }

  function formatArgonMemory(memoryKib: number): string {
    if (memoryKib <= 0) {
      return '0 MB';
    }
    const memoryMb = memoryKib / 1024;
    if (memoryMb >= 1024) {
      return `${gigabyteFormatter.format(memoryMb / 1024)} GB`;
    }
    return `${memoryFormatter.format(memoryMb)} MB`;
  }

  function openPasswordModal() {
    passwordModalOpen = true;
    currentPassword = '';
    newPassword = '';
    confirmPassword = '';
    changePasswordError = '';
    showCurrentPassword = false;
  }

  function closePasswordModal() {
    passwordModalOpen = false;
    currentPassword = '';
    newPassword = '';
    confirmPassword = '';
    changePasswordError = '';
    showCurrentPassword = false;
  }

  function handleDialogChange(open: boolean) {
    passwordModalOpen = open;
    if (!passwordModalOpen) {
      changePasswordError = '';
      showCurrentPassword = false;
    }
  }

  function togglePasswordVisibility() {
    showCurrentPassword = !showCurrentPassword;
  }

  function openKdfModal() {
    kdfModalOpen = true;
    kdfCurrentPassword = '';
    showKdfPassword = false;
    kdfError = '';
    kdfMemoryMb = String(Math.max(8, Math.round(argon2Params.memoryKib / 1024)));
    kdfTimeCost = String(argon2Params.timeCost);
    kdfParallelism = String(argon2Params.parallelism);
  }

  function handleKdfDialogChange(open: boolean) {
    kdfModalOpen = open;
    if (!open) {
      kdfCurrentPassword = '';
      kdfError = '';
      showKdfPassword = false;
    }
  }

  function toggleKdfPasswordVisibility() {
    showKdfPassword = !showKdfPassword;
  }

  async function loadArgon2Params() {
    argon2Loading = true;
    try {
      const argon2Result = await callBackend<Argon2Params>('get_argon2_params');
      if (argon2Result) {
        argon2Params = { ...argon2Result };
      }
    } catch (error) {
      toast.error(`Failed to load key derivation settings: ${parseError(error)}`);
    } finally {
      argon2Loading = false;
    }
  }

  async function submitPasswordChange() {
    changePasswordError = '';
    if (!isPasswordFormValid) {
      if (newPassword.trim() !== confirmPassword.trim()) {
        changePasswordError = 'New password and confirmation do not match.';
      } else if (newPassword.trim().length < MIN_PASSWORD_LENGTH) {
        changePasswordError = `New password must be at least ${MIN_PASSWORD_LENGTH} characters.`;
      } else if (newPassword.trim() === currentPassword.trim()) {
        changePasswordError = 'New password must be different from the current password.';
      }
      return;
    }

    isChangingPassword = true;
    try {
      await callBackend('rotate_master_password', {
        currentPassword,
        newPassword
      });
      toast.success('Master password updated successfully.');
      closePasswordModal();
      await loadArgon2Params();
    } catch (error) {
      toast.error(`Failed to update master password: ${parseError(error)}`);
    } finally {
      isChangingPassword = false;
    }
  }

  async function submitKdfUpdate() {
    kdfError = '';
    if (!isKdfFormValid) {
      const memoryMb = Number(kdfMemoryMb);
      const timeCost = Number(kdfTimeCost);
      const parallelism = Number(kdfParallelism);
      if (kdfCurrentPassword.trim().length === 0) {
        kdfError = 'Current password is required.';
      } else if (memoryMb < 8) {
        kdfError = 'Memory must be at least 8 MiB.';
      } else if (timeCost < 1) {
        kdfError = 'Time cost must be at least 1.';
      } else if (parallelism < 1) {
        kdfError = 'Parallelism must be at least 1.';
      }
      return;
    }

    const memoryMb = Number(kdfMemoryMb);
    const timeCost = Number(kdfTimeCost);
    const parallelism = Number(kdfParallelism);
    isUpdatingKdf = true;
    try {
      await callBackend('update_argon2_params', {
        currentPassword: kdfCurrentPassword,
        memoryKib: Math.round(memoryMb * 1024),
        timeCost: Math.round(timeCost),
        parallelism: Math.round(parallelism)
      });
      toast.success('Key derivation parameters updated.');
      handleKdfDialogChange(false);
      await loadArgon2Params();
    } catch (error) {
      toast.error(`Failed to update key derivation parameters: ${parseError(error)}`);
    } finally {
      isUpdatingKdf = false;
    }
  }

  async function performMemoryWipe() {
    securityActionPending = { ...securityActionPending, 'wipe-memory': true };
    try {
      await callBackend('wipe_memory');
      toast.success('Sensitive data cleared from memory. The vault has been locked.');
    } catch (error) {
      toast.error(`Failed to clear memory: ${parseError(error)}`);
    } finally {
      securityActionPending = { ...securityActionPending, 'wipe-memory': false };
    }
  }

  async function runIntegrityCheck() {
    securityActionPending = { ...securityActionPending, 'integrity-check': true };
    try {
      const result = await callBackend<string>('run_integrity_check');
      if (result.trim().toLowerCase() === 'ok') {
        toast.success('Vault integrity check completed successfully.');
      } else {
        toast.error(`Integrity check reported issues: ${result}`);
      }
    } catch (error) {
      toast.error(`Failed to run integrity check: ${parseError(error)}`);
    } finally {
      securityActionPending = { ...securityActionPending, 'integrity-check': false };
    }
  }

  async function handleSecurityAction(actionId: SecurityActionId) {
    if (actionId === 'rekey') {
      openPasswordModal();
      return;
    }

    if (actionId === 'wipe-memory') {
      await performMemoryWipe();
    } else if (actionId === 'integrity-check') {
      await runIntegrityCheck();
    }
  }

  const argon2Summary = $derived(
    `Argon2id • memory ${formatArgonMemory(argon2Params.memoryKib)} • time cost ${argon2Params.timeCost} • parallelism ${argon2Params.parallelism}`
  );

  const isPasswordFormValid = $derived(
    currentPassword.length > 0 && newPassword.length >= 8 && newPassword === confirmPassword
  );

  const kdfMemoryMbValue = $derived(Number(kdfMemoryMb));
  const kdfTimeCostValue = $derived(Number(kdfTimeCost));
  const kdfParallelismValue = $derived(Number(kdfParallelism));

  const isKdfFormValid = $derived(
    kdfCurrentPassword.length > 0 &&
      kdfMemoryMbValue >= 8 &&
      kdfTimeCostValue >= 1 &&
      kdfParallelismValue >= 1
  );

  const privacyToggles = [
    { key: 'externalBreachCheck' as keyof SecuritySettings, title: 'External Breach Check' },
    { key: 'localReuseDetection' as keyof SecuritySettings, title: 'Local Reuse Detection' },
    { key: 'secureRAMHandling' as keyof SecuritySettings, title: 'Secure RAM Handling' }
  ];

  const securityActions = [
    { id: 'rekey' as SecurityActionId, Icon: KeyRound },
    { id: 'wipe-memory' as SecurityActionId, Icon: Trash2 },
    { id: 'integrity-check' as SecurityActionId, Icon: ShieldCheck }
  ];

  const autoLockOptions = [
    { value: '1 minute', label: '1 minute' },
    { value: '5 minutes', label: '5 minutes' },
    { value: '15 minutes', label: '15 minutes' },
    { value: '30 minutes', label: '30 minutes' },
    { value: '1 hour', label: '1 hour' },
    { value: 'Never', label: 'Never' }
  ];

  const lockGraceOptions = [
    { value: '0', label: '0 seconds' },
    { value: '5', label: '5 seconds' },
    { value: '10', label: '10 seconds' },
    { value: '30', label: '30 seconds' }
  ];

  const clipboardClearOptions = [
    { value: '5', label: '5 seconds' },
    { value: '10', label: '10 seconds' },
    { value: '15', label: '15 seconds' },
    { value: '30', label: '30 seconds' },
    { value: '60', label: '1 minute' },
    { value: '120', label: '2 minutes' },
    { value: '0', label: 'Never' }
  ];

  function getAutoLockLabel(value: string): string {
    const opt = autoLockOptions.find((o) => o.value === value);
    return opt ? opt.label : value;
  }
</script>

<div class="min-h-0 flex-1 space-y-6 px-6 py-8">
  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div class="flex w-full items-center justify-between">
        <div>
          <CardTitle>{t('Vault Health')}</CardTitle>
          <CardDescription>
            {t('Security analysis of your stored items.')}
          </CardDescription>
        </div>
        <Button variant="ghost" size="sm" onclick={loadSecurityReport} disabled={healthLoading}>
          <RefreshCw class={cn('h-4 w-4', healthLoading && 'animate-spin')} />
        </Button>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      {#if healthLoading && !healthReport}
        <div class="flex items-center justify-center py-8">
          <Spinner class="text-primary/40 h-8 w-8" />
        </div>
      {:else if healthError}
        <Alert variant="destructive">
          <CircleAlert class="h-4 w-4" />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{healthError}</AlertDescription>
        </Alert>
      {:else if healthReport}
        {@const reusedCount = healthReport.reusedPasswords.length}
        {@const weakCount = healthReport.weakPasswords.length}
        {@const breachedCount = healthReport.breachedPasswords.length}

        <div class="grid gap-4 sm:grid-cols-3">
          <div
            class={cn(
              'rounded-lg border p-4',
              reusedCount > 0
                ? 'border-destructive/40 bg-destructive/5'
                : 'border-border/60 bg-muted/20'
            )}
          >
            <p class="text-sm font-semibold">
              {t('Reused Passwords')}
            </p>
            <p
              class={cn(
                'mt-1 text-2xl font-bold',
                reusedCount > 0 ? 'text-destructive' : 'text-foreground'
              )}
            >
              {reusedCount}
            </p>
            <p class="text-muted-foreground mt-1 text-xs">
              {reusedCount > 0
                ? t('Multiple items share the same password.')
                : t('No password reuse detected.')}
            </p>
          </div>

          <div
            class={cn(
              'rounded-lg border p-4',
              weakCount > 0 ? 'border-warning/40 bg-warning/5' : 'border-border/60 bg-muted/20'
            )}
          >
            <p class="text-sm font-semibold">{t('Weak Passwords')}</p>
            <p
              class={cn(
                'mt-1 text-2xl font-bold',
                weakCount > 0 ? 'text-warning-foreground' : 'text-foreground'
              )}
            >
              {weakCount}
            </p>
            <p class="text-muted-foreground mt-1 text-xs">
              {weakCount > 0
                ? t('Passwords with low security score.')
                : t('No weak passwords detected.')}
            </p>
          </div>

          <div
            class={cn(
              'rounded-lg border p-4',
              breachedCount > 0
                ? 'border-destructive/40 bg-destructive/5'
                : 'border-border/60 bg-muted/20'
            )}
          >
            <p class="text-sm font-semibold">{t('Breached Passwords')}</p>
            <p
              class={cn(
                'mt-1 text-2xl font-bold',
                breachedCount > 0 ? 'text-destructive' : 'text-foreground'
              )}
            >
              {breachedCount}
            </p>
            <p class="text-muted-foreground mt-1 text-xs">
              {breachedCount > 0
                ? t('Known compromised credentials.')
                : t('No breached passwords detected.')}
            </p>
          </div>
        </div>

        {#if reusedCount > 0 || weakCount > 0 || breachedCount > 0}
          <div
            class="border-destructive/40 bg-destructive/10 text-destructive mt-4 flex items-start gap-3 rounded-lg border p-3 text-sm"
          >
            <TriangleAlert class="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
            <p>
              {t(
                'Security issues detected. Consider updating shared or short passwords to improve vault integrity.'
              )}
            </p>
          </div>

          <div class="mt-6 space-y-6">
            {#if reusedCount > 0}
              <div class="space-y-3">
                <h4 class="flex items-center gap-2 text-sm font-semibold">
                  <RefreshCw class="text-destructive h-4 w-4" />
                  {t('Reused Password Groups')}
                </h4>
                <div class="grid gap-3">
                  {#each healthReport.reusedPasswords as group (group.passwordHash)}
                    <div class="border-border/40 bg-muted/10 rounded-lg border p-3">
                      <div class="mb-2 flex items-center justify-between">
                        <span class="text-muted-foreground font-mono text-xs">
                          {group.passwordHash.slice(0, 8)}...
                        </span>
                        <Badge variant="outline" class="text-[10px]">{group.count} items</Badge>
                      </div>
                      <div class="space-y-2">
                        {#each group.itemIds as itemId (itemId)}
                          {@const item = getProblematicItem(itemId)}
                          {#if item}
                            <div class="group flex items-center justify-between">
                              <span class="text-sm">{item.title}</span>
                              <Button
                                variant="ghost"
                                size="sm"
                                class="h-7 px-2 opacity-0 transition-opacity group-hover:opacity-100"
                                onclick={() => navigateToItem(item.id)}
                              >
                                {t('View')}
                              </Button>
                            </div>
                          {/if}
                        {/each}
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            {#if weakCount > 0}
              <div class="space-y-3">
                <h4 class="flex items-center gap-2 text-sm font-semibold">
                  <TriangleAlert class="text-warning-foreground h-4 w-4" />
                  {t('Weak Passwords')}
                </h4>
                <div class="border-border/40 divide-border/40 divide-y rounded-lg border">
                  {#each healthReport.weakPasswords as itemId (itemId)}
                    {@const item = getProblematicItem(itemId)}
                    {#if item}
                      <div class="group flex items-center justify-between p-3">
                        <span class="text-sm">{item.title}</span>
                        <Button
                          variant="ghost"
                          size="sm"
                          class="h-7 px-2 opacity-0 transition-opacity group-hover:opacity-100"
                          onclick={() => navigateToItem(item.id)}
                        >
                          {t('View')}
                        </Button>
                      </div>
                    {/if}
                  {/each}
                </div>
              </div>
            {/if}

            {#if breachedCount > 0}
              <div class="space-y-3">
                <h4 class="flex items-center gap-2 text-sm font-semibold">
                  <ShieldCheck class="text-destructive h-4 w-4" />
                  {t('Breached Passwords')}
                </h4>
                <div class="border-border/40 divide-border/40 divide-y rounded-lg border">
                  {#each healthReport.breachedPasswords as itemId (itemId)}
                    {@const item = getProblematicItem(itemId)}
                    {#if item}
                      <div class="group flex items-center justify-between p-3">
                        <span class="text-sm">{item.title}</span>
                        <Button
                          variant="ghost"
                          size="sm"
                          class="h-7 px-2 opacity-0 transition-opacity group-hover:opacity-100"
                          onclick={() => navigateToItem(item.id)}
                        >
                          {t('View')}
                        </Button>
                      </div>
                    {/if}
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {:else}
          <div
            class="mt-4 flex items-start gap-3 rounded-lg border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm text-emerald-600"
          >
            <Check class="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
            <p>{t('Your vault health looks great!')}</p>
          </div>
        {/if}
      {/if}
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Two-factor authentication')}
        </CardTitle>
        <CardDescription>
          {t('Protect vault unlocks with a time-based one-time password.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-5 pt-4">
      {#if totpSetupSuccess}
        <Alert>
          <Check class="text-primary h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>
              {t('Authenticator enabled')}
            </AlertTitle>
            <AlertDescription>{totpSetupSuccess}</AlertDescription>
          </div>
        </Alert>
      {/if}

      {#if totpDisableSuccess}
        <Alert>
          <ShieldCheck class="text-primary h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>
              {t('Authenticator disabled')}
            </AlertTitle>
            <AlertDescription>{totpDisableSuccess}</AlertDescription>
          </div>
        </Alert>
      {/if}

      {#if totpStatusError}
        <Alert variant="destructive">
          <CircleAlert class="h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>
              {t('Unable to load status')}
            </AlertTitle>
            <AlertDescription>{totpStatusError}</AlertDescription>
          </div>
        </Alert>
      {/if}

      {#if totpGenerationError && !pendingTotpSecret}
        <Alert variant="destructive">
          <CircleAlert class="h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>
              {t('Unable to generate secret')}
            </AlertTitle>
            <AlertDescription>{totpGenerationError}</AlertDescription>
          </div>
        </Alert>
      {/if}

      <div class="border-border/60 bg-muted/20 space-y-3 rounded-lg border p-4">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p class="text-foreground text-sm font-semibold">
              {t('Current status')}
            </p>
            <p class="text-muted-foreground text-sm">
              {loginTotpStore.configured
                ? t('Unlocking requires both your master password and an authenticator token.')
                : t(
                    'Generate a secret to require an authenticator token when unlocking the vault.'
                  )}
            </p>
          </div>
          <Badge
            class={cn(
              'px-3 py-1 text-xs font-medium',
              loginTotpStore.configured
                ? 'bg-emerald-500/15 text-emerald-500'
                : 'bg-muted text-muted-foreground'
            )}
          >
            {loginTotpStore.configured ? 'Enabled' : 'Disabled'}
          </Badge>
        </div>

        <div class="flex flex-wrap gap-2">
          <Button
            type="button"
            class="gap-2"
            onclick={handleGenerateTotpSecret}
            disabled={isGeneratingTotpSecret || isTotpStatusLoading}
          >
            {#if isGeneratingTotpSecret}
              <Spinner class="h-4 w-4" aria-hidden="true" />
            {:else}
              <QrCode class="h-4 w-4" aria-hidden="true" />
            {/if}
            {generateButtonLabel}
          </Button>

          {#if loginTotpStore.configured}
            <Button
              type="button"
              variant="outline"
              class="gap-2"
              onclick={handleDisableTotp}
              disabled={isDisablingTotp}
            >
              {#if isDisablingTotp}
                <Spinner class="h-4 w-4" aria-hidden="true" />
              {:else}
                <ShieldCheck class="h-4 w-4" aria-hidden="true" />
              {/if}
              Disable TOTP
            </Button>
          {/if}

          <Button
            type="button"
            variant="ghost"
            class="gap-2"
            onclick={refreshTotpStatus}
            disabled={isTotpStatusLoading}
          >
            <RefreshCw
              class={`h-4 w-4 ${isTotpStatusLoading ? 'animate-spin' : ''}`}
              aria-hidden="true"
            />
            Refresh status
          </Button>
        </div>

        {#if isTotpStatusLoading}
          <p class="text-muted-foreground flex items-center gap-2 text-xs">
            <Spinner class="h-3 w-3" aria-hidden="true" />
            Checking configuration…
          </p>
        {/if}
      </div>

      {#if pendingTotpSecret}
        <div class="border-primary/40 bg-primary/5 space-y-4 rounded-lg border p-4">
          <div class="flex items-center gap-2">
            <QrCode class="text-primary h-5 w-5" aria-hidden="true" />
            <div>
              <p class="text-foreground text-sm font-semibold">
                Step 1 — Add the secret to your authenticator
              </p>
              <p class="text-muted-foreground text-sm">
                Scan the QR code or copy the secret, then confirm a code to finish enabling TOTP.
              </p>
            </div>
          </div>

          <div class="border-border/60 bg-background rounded-md border p-4">
            <p class="text-muted-foreground text-xs font-semibold tracking-wide uppercase">
              Secret
            </p>
            <p class="text-foreground mt-2 font-mono text-lg wrap-break-word">
              {formatSecret(pendingTotpSecret)}
            </p>
            <div class="mt-3 flex flex-wrap gap-2">
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={() => copySecret(pendingTotpSecret, 'pending')}
              >
                <Copy class="h-4 w-4" aria-hidden="true" />
                Copy secret
              </Button>
              <Button
                type="button"
                variant="outline"
                class="gap-2"
                onclick={() => copyProvisioningUri(pendingProvisioningUri, 'pending')}
                disabled={!pendingProvisioningUri}
              >
                <Link2 class="h-4 w-4" aria-hidden="true" />
                Copy setup link
              </Button>
            </div>
            {#if secretCopyFeedback?.context === 'pending'}
              <p
                class={`mt-2 text-xs ${secretCopyFeedback.variant === 'success' ? 'text-primary' : 'text-destructive'}`}
                aria-live="polite"
              >
                {secretCopyFeedback.message}
              </p>
            {/if}
            {#if uriCopyFeedback?.context === 'pending'}
              <p
                class={`mt-1 text-xs ${uriCopyFeedback.variant === 'success' ? 'text-primary' : 'text-destructive'}`}
                aria-live="polite"
              >
                {uriCopyFeedback.message}
              </p>
            {/if}
          </div>

          <div class="space-y-2">
            <Label for="totp-verification">
              {t('Step 2 — Confirm a code')}
            </Label>
            <Input
              id="totp-verification"
              type="number"
              maxlength={TOTP_CODE_LENGTH}
              autocomplete="one-time-code"
              placeholder={t('Enter 6-digit code')}
              bind:inputValue={totpVerificationCode}
              oninput={sanitizeTotpCode}
              title="Verification Code"
            />
            {#if totpVerificationError}
              <p class="text-destructive text-sm">{totpVerificationError}</p>
            {/if}
          </div>

          {#if totpGenerationError}
            <Alert variant="destructive">
              <CircleAlert class="h-4 w-4" aria-hidden="true" />
              <div class="space-y-1">
                <AlertTitle>
                  {t('Unable to generate secret')}
                </AlertTitle>
                <AlertDescription>{totpGenerationError}</AlertDescription>
              </div>
            </Alert>
          {/if}

          <div class="flex flex-wrap gap-2">
            <Button
              type="button"
              class="gap-2"
              onclick={handleConfirmTotp}
              disabled={isConfirmingTotp || totpVerificationCode.length !== TOTP_CODE_LENGTH}
            >
              {#if isConfirmingTotp}
                <Spinner class="h-4 w-4" aria-hidden="true" />
              {:else}
                <Check class="h-4 w-4" aria-hidden="true" />
              {/if}
              {isConfirmingTotp ? t('Verifying…') : t('Verify & enable')}
            </Button>
            <Button
              type="button"
              variant="outline"
              class="gap-2"
              onclick={handleGenerateTotpSecret}
              disabled={isGeneratingTotpSecret}
            >
              <RefreshCw
                class={`h-4 w-4 ${isGeneratingTotpSecret ? 'animate-spin' : ''}`}
                aria-hidden="true"
              />
              {t('Generate another secret')}
            </Button>
            <Button type="button" variant="ghost" onclick={cancelTotpSetup}>
              {t('Cancel')}
            </Button>
          </div>

          <p class="text-muted-foreground text-xs">
            {t(
              'Codes rotate every 30 seconds. If verification fails, wait for the next code before trying again.'
            )}
          </p>
        </div>
      {/if}

      {#if loginTotpStore.configured && !pendingTotpSecret}
        {@const storedSecret = loginTotpStore.secret}
        <div class="border-border/60 bg-muted/10 space-y-3 rounded-lg border p-4">
          <div class="flex items-center gap-2">
            <ShieldCheck class="text-primary h-5 w-5" aria-hidden="true" />
            <div>
              <p class="text-foreground text-sm font-semibold">Stored secret on this device</p>
              <p class="text-muted-foreground text-sm">
                {storedSecret
                  ? t(
                      'Copy the secret if you need to enrol another authenticator or keep an offline backup.'
                    )
                  : t(
                      'This device does not have a local copy of the secret. Rotate the secret to capture it again.'
                    )}
              </p>
            </div>
          </div>

          {#if storedSecret}
            <div class="border-border/60 bg-background rounded-md border p-4">
              <p class="text-muted-foreground text-xs font-semibold tracking-wide uppercase">
                Secret
              </p>
              <p class="text-foreground mt-2 font-mono text-lg wrap-break-word">
                {formatSecret(storedSecret)}
              </p>
              <div class="mt-3 flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="outline"
                  class="gap-2"
                  onclick={() => copySecret(storedSecret, 'stored')}
                >
                  <Copy class="h-4 w-4" aria-hidden="true" />
                  {t('Copy secret')}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  class="gap-2"
                  onclick={() => copyProvisioningUri(currentProvisioningUri, 'stored')}
                  disabled={!currentProvisioningUri}
                >
                  <Link2 class="h-4 w-4" aria-hidden="true" />
                  {t('Copy setup link')}
                </Button>
              </div>
              {#if secretCopyFeedback?.context === 'stored'}
                <p
                  class={`mt-2 text-xs ${secretCopyFeedback.variant === 'success' ? 'text-primary' : 'text-destructive'}`}
                  aria-live="polite"
                >
                  {secretCopyFeedback.message}
                </p>
              {/if}
              {#if uriCopyFeedback?.context === 'stored'}
                <p
                  class={`mt-1 text-xs ${uriCopyFeedback.variant === 'success' ? 'text-primary' : 'text-destructive'}`}
                  aria-live="polite"
                >
                  {uriCopyFeedback.message}
                </p>
              {/if}
            </div>
          {:else}
            <Alert>
              <CircleAlert class="h-4 w-4" aria-hidden="true" />
              <div class="space-y-1">
                <AlertTitle>
                  {t('No local secret available')}
                </AlertTitle>
                <AlertDescription>
                  {t('Rotate the secret to store a copy on this device for backup access.')}
                </AlertDescription>
              </div>
            </Alert>
          {/if}
        </div>
      {/if}
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <Lock class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Master Password & Encryption')}
        </CardTitle>
        <CardDescription>
          {t('Manage the master password and key derivation policy.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-5 pt-4">
      <div
        class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t('Master Password')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Update the password used to unlock your vault.')}
          </p>
        </div>
        <Button variant="outline" onclick={openPasswordModal}>
          {t('Change Password')}
        </Button>
      </div>

      <div
        class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t('Key Derivation')}
          </p>
          <p class="text-muted-foreground text-sm">
            {argon2Loading ? t('Loading key derivation parameters…') : argon2Summary}
          </p>
        </div>
        <Button variant="outline" size="sm" onclick={openKdfModal}>
          {t('Reconfigure KDF')}
        </Button>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <CalendarClock class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Auto-lock Controls')}
        </CardTitle>
        <CardDescription>
          {t('Define when the vault should automatically lock itself.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t('Lock on Suspend')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Lock whenever the system sleeps or hibernates.')}
          </p>
        </div>
        <Switch
          checked={currentSettings.lockOnSuspend}
          aria-label="Toggle lock on suspend"
          onCheckedChange={() => toggleSetting('lockOnSuspend')}
        />
      </div>

      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t('Lock on Minimise')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Lock the vault when the window is minimised.')}
          </p>
        </div>
        <Switch
          checked={currentSettings.lockOnMinimize}
          aria-label="Toggle lock on minimise"
          onCheckedChange={() => toggleSetting('lockOnMinimize')}
        />
      </div>

      <div class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4">
        <Label class="text-foreground text-sm font-semibold">
          {t('Lock Grace Period')}
        </Label>
        <p class="text-muted-foreground text-sm">
          {t('Delay before locking after minimize or suspend.')}
        </p>
        <Select
          type="single"
          value={currentSettings.lockGraceSeconds.toString()}
          onValueChange={updateLockGrace}
        >
          <SelectTrigger aria-label="Select lock grace period" class="w-full sm:w-56">
            <span data-slot="select-value" class="truncate text-sm">
              {lockGraceOptions.find((o) => o.value === currentSettings.lockGraceSeconds.toString())
                ?.label || t('Select duration')}
            </span>
          </SelectTrigger>
          <SelectContent>
            {#each lockGraceOptions as option (option.value)}
              <SelectItem value={option.value}>
                {option.label}
              </SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>

      <div class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4">
        <Label class="text-foreground text-sm font-semibold">
          {t('Auto-lock After Inactivity')}
        </Label>
        <p class="text-muted-foreground text-sm">
          {t('Lock the vault automatically after the selected idle period.')}
        </p>
        <Select
          type="single"
          value={currentSettings.autoLockInactivity}
          onValueChange={updateAutoLock}
        >
          <SelectTrigger aria-label="Select auto-lock inactivity" class="w-full sm:w-56">
            <span data-slot="select-value" class="truncate text-sm">
              {getAutoLockLabel(currentSettings.autoLockInactivity) || t('Select duration')}
            </span>
          </SelectTrigger>
          <SelectContent>
            {#each autoLockOptions as option (option.value)}
              <SelectItem value={option.value}>
                {getAutoLockLabel(option.value)}
              </SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>

      <div class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4">
        <Label class="text-foreground text-sm font-semibold">
          {t('Clipboard Clear Timeout')}
        </Label>
        <p class="text-muted-foreground text-sm">
          {t('Automatically clear sensitive data from your clipboard after the selected duration.')}
        </p>
        <Select
          type="single"
          value={currentClipboardSettings.clearAfterDuration.toString()}
          onValueChange={updateClipboardClear}
        >
          <SelectTrigger aria-label="Select clipboard clear timeout" class="w-full sm:w-56">
            <span data-slot="select-value" class="truncate text-sm">
              {clipboardClearOptions.find(
                (o) => o.value === currentClipboardSettings.clearAfterDuration.toString()
              )?.label || t('Select duration')}
            </span>
          </SelectTrigger>
          <SelectContent>
            {#each clipboardClearOptions as option (option.value)}
              <SelectItem value={option.value}>
                {option.label}
              </SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <FingerprintPattern class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Biometric & Session')}
        </CardTitle>
        <CardDescription>
          {t('Control biometric unlock availability and session persistence.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t('Biometric Unlock')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Allow fingerprint or face recognition to unlock the vault.')}
          </p>
        </div>
        <Switch
          checked={isBiometricsEnabled}
          disabled={isBiometricActionLoading}
          aria-label="Toggle biometric unlock"
          onCheckedChange={handleBiometricToggle}
        />
      </div>

      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t('Session Persistence')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t('Remember the unlocked session between restarts.')}
          </p>
        </div>
        <Switch
          checked={currentSettings.sessionPersistence}
          aria-label="Toggle session persistence"
          onCheckedChange={() => toggleSetting('sessionPersistence')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <EyeOff class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Privacy Controls')}
        </CardTitle>
        <CardDescription>
          {t('Fine-tune privacy and diagnostic data handling.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-5 pt-4">
      <div class="flex flex-col gap-3">
        {#each privacyToggles as toggle (toggle.key)}
          <div
            class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
          >
            <div>
              <p class="text-foreground text-sm font-semibold">
                {toggle.key === 'externalBreachCheck'
                  ? t('External Breach Check')
                  : toggle.key === 'localReuseDetection'
                    ? t('Local Reuse Detection')
                    : t('Secure RAM Handling')}
              </p>
              <p class="text-muted-foreground text-sm">
                {toggle.key === 'externalBreachCheck'
                  ? t('Cross-reference vault items against known breach databases.')
                  : toggle.key === 'localReuseDetection'
                    ? t('Alert when passwords repeat across vault entries.')
                    : t('Allocate hardened memory regions for sensitive operations.')}
              </p>
            </div>
            <Switch
              checked={currentSettings[toggle.key] as boolean}
              aria-label={`Toggle ${toggle.title}`}
              onCheckedChange={() => toggleSetting(toggle.key)}
            />
          </div>
        {/each}
      </div>

      <div class="grid gap-3 sm:grid-cols-2">
        <Button
          variant="outline"
          class="justify-start gap-3"
          onclick={async () => {
            try {
              await callBackend('open_app_data_folder');
            } catch (_error) {
              toast.error(t('Failed to open app data folder'));
            }
          }}
        >
          <HardDrive class="h-4 w-4" aria-hidden="true" />
          {t('Access Local Logs')}
        </Button>
        <Button
          variant="outline"
          class="justify-start gap-3"
          onclick={async () => {
            try {
              await callBackend('clear_app_logs');
              toast.success(t('Logs cleared successfully'));
            } catch (_error) {
              toast.error(t('Failed to clear logs'));
            }
          }}
        >
          <Trash2 class="h-4 w-4" aria-hidden="true" />
          {t('Clear Local Logs')}
        </Button>
      </div>

      <div class="border-border/60 bg-muted/10 rounded-lg border p-4">
        <p class="text-foreground text-sm font-semibold">Data Retention Summary</p>
        <ul class="text-muted-foreground mt-2 list-disc space-y-1 pl-5 text-sm">
          <li>Vault data remains encrypted locally and is never uploaded.</li>
          <li>Activity logs rotate every 30 days unless cleared sooner.</li>
          <li>Crash reports are anonymised and retained for up to 90 days.</li>
        </ul>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t('Security Actions')}
        </CardTitle>
        <CardDescription>
          {t('Execute advanced maintenance and security tasks.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="grid gap-3 md:grid-cols-3">
        {#each securityActions as action (action.id)}
          <Button
            type="button"
            variant="outline"
            class="border-border/60 bg-muted/20 hover:border-primary/50 hover:text-primary h-full w-full flex-col items-start gap-3 rounded-xl p-4 text-left transition-colors"
            onclick={() => handleSecurityAction(action.id)}
            disabled={action.id !== 'rekey' && securityActionPending[action.id]}
            aria-busy={action.id !== 'rekey' && securityActionPending[action.id]}
          >
            {#if action.id !== 'rekey' && securityActionPending[action.id]}
              <Spinner class="text-primary h-5 w-5" aria-hidden="true" />
            {:else}
              <action.Icon class="text-primary h-5 w-5" aria-hidden="true" />
            {/if}
            <div class="space-y-1">
              <p class="text-foreground text-sm font-semibold">
                {action.id === 'rekey'
                  ? t('Re-key Vault')
                  : action.id === 'wipe-memory'
                    ? t('Clear Memory')
                    : t('Integrity Check')}
              </p>
              <p class="text-muted-foreground text-xs wrap-break-word whitespace-normal">
                {action.id === 'rekey'
                  ? t('Rotate encryption keys and re-encrypt stored data.')
                  : action.id === 'wipe-memory'
                    ? t('Scrub sensitive material from memory immediately.')
                    : t('Verify vault contents for tampering or corruption.')}
              </p>
            </div>
          </Button>
        {/each}
      </div>
    </CardContent>
  </Card>
</div>

<Dialog open={passwordModalOpen} onOpenChange={handleDialogChange}>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>
        {t('Change Master Password')}
      </DialogTitle>
      <DialogDescription>
        {t(
          'Provide your current master password and enter a new secure password to re-encrypt the vault.'
        )}
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="current-password">
          {t('Current Password')}
        </Label>
        <div class="relative">
          <Input
            id="current-password"
            type={showCurrentPassword ? 'text' : 'password'}
            placeholder={t('Enter current password')}
            bind:inputValue={currentPassword}
            title="Current Password"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute top-1/2 right-1 -translate-y-1/2"
            aria-label={showCurrentPassword
              ? t('Hide current password')
              : t('Show current password')}
            onclick={togglePasswordVisibility}
          >
            {#if showCurrentPassword}
              <EyeOff class="text-primary h-4 w-4" aria-hidden="true" />
            {:else}
              <Eye class="text-muted-foreground h-4 w-4" aria-hidden="true" />
            {/if}
          </Button>
        </div>
      </div>

      <div class="space-y-2">
        <Label for="new-password">{t('New Password')}</Label>
        <Input
          id="new-password"
          type="password"
          placeholder={t('Enter new password')}
          bind:inputValue={newPassword}
          title="New Password"
        />
      </div>

      <div class="space-y-2">
        <Label for="confirm-password">
          {t('Confirm New Password')}
        </Label>
        <Input
          id="confirm-password"
          type="password"
          placeholder={t('Confirm new password')}
          bind:inputValue={confirmPassword}
          title="Confirm Password"
        />
      </div>

      <div
        class="border-destructive/40 bg-destructive/10 text-destructive flex items-start gap-3 rounded-lg border p-3 text-sm"
      >
        <TriangleAlert class="mt-0.5 h-4 w-4" aria-hidden="true" />
        <p>
          {t(
            'Changing the master password re-encrypts the vault. The operation may take several minutes for large vaults.'
          )}
        </p>
      </div>
      {#if changePasswordError}
        <p class="text-destructive text-sm">{changePasswordError}</p>
      {/if}
    </div>

    <DialogFooter class="gap-2">
      <Button type="button" variant="outline" onclick={closePasswordModal}>
        {t('Cancel')}
      </Button>
      <Button
        type="button"
        variant="destructive"
        onclick={submitPasswordChange}
        disabled={!isPasswordFormValid || isChangingPassword}
        aria-busy={isChangingPassword}
      >
        {#if isChangingPassword}
          <Spinner class="mr-2 h-4 w-4" aria-hidden="true" />
        {/if}
        {t('Change Password')}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog open={kdfModalOpen} onOpenChange={handleKdfDialogChange}>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>
        {t('Reconfigure Key Derivation')}
      </DialogTitle>
      <DialogDescription>
        {t('Adjust the Argon2id parameters used when deriving the vault encryption key.')}
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      <div class="grid gap-3 sm:grid-cols-3">
        <div class="space-y-2">
          <Label for="kdf-memory">{t('Memory (MiB)')}</Label>
          <Input
            id="kdf-memory"
            type="number"
            min="8"
            bind:inputValue={kdfMemoryMb}
            title="Memory"
          />
        </div>
        <div class="space-y-2">
          <Label for="kdf-time">{t('Time Cost')}</Label>
          <Input
            id="kdf-time"
            type="number"
            min="1"
            bind:inputValue={kdfTimeCost}
            title="Time Cost"
          />
        </div>
        <div class="space-y-2">
          <Label for="kdf-parallelism">{t('Parallelism')}</Label>
          <Input
            id="kdf-parallelism"
            type="number"
            min="1"
            bind:inputValue={kdfParallelism}
            title="Parallelism"
          />
        </div>
      </div>

      <div class="space-y-2">
        <Label for="kdf-password">{t('Current Password')}</Label>
        <div class="relative">
          <Input
            id="kdf-password"
            type={showKdfPassword ? 'text' : 'password'}
            placeholder={t('Enter current password')}
            bind:inputValue={kdfCurrentPassword}
            title="Password"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute top-1/2 right-1 -translate-y-1/2"
            aria-label={showKdfPassword ? t('Hide current password') : t('Show current password')}
            onclick={toggleKdfPasswordVisibility}
          >
            {#if showKdfPassword}
              <EyeOff class="text-primary h-4 w-4" aria-hidden="true" />
            {:else}
              <Eye class="text-muted-foreground h-4 w-4" aria-hidden="true" />
            {/if}
          </Button>
        </div>
      </div>

      <div
        class="border-border/60 bg-muted/20 text-muted-foreground flex items-start gap-3 rounded-lg border p-3 text-sm"
      >
        <TriangleAlert class="text-primary mt-0.5 h-4 w-4" aria-hidden="true" />
        <p>
          {t('Updating Argon2 parameters will re-encrypt the vault and may take a few moments.')}
        </p>
      </div>

      {#if kdfError}
        <p class="text-destructive text-sm">{kdfError}</p>
      {/if}
    </div>

    <DialogFooter class="gap-2">
      <Button type="button" variant="outline" onclick={() => handleKdfDialogChange(false)}>
        {t('Cancel')}
      </Button>
      <Button
        type="button"
        variant="destructive"
        onclick={submitKdfUpdate}
        disabled={!isKdfFormValid || isUpdatingKdf}
        aria-busy={isUpdatingKdf}
      >
        {#if isUpdatingKdf}
          <Spinner class="mr-2 h-4 w-4" aria-hidden="true" />
        {/if}
        {t('Update Parameters')}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog open={biometricModalOpen} onOpenChange={handleBiometricDialogChange}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>Enable Biometric Unlock</DialogTitle>
      <DialogDescription>
        Enter your master password to securely store the encryption key in your system's keychain.
      </DialogDescription>
    </DialogHeader>
    <div class="space-y-4 py-2">
      <div class="space-y-2">
        <Label for="bio-password">Master Password</Label>
        <div class="relative">
          <Input
            id="bio-password"
            type={showBiometricPassword ? 'text' : 'password'}
            placeholder="Enter master password"
            bind:inputValue={biometricPassword}
            onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && submitBiometricEnable()}
            title="Password"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute top-1/2 right-1 -translate-y-1/2"
            onclick={toggleBiometricPasswordVisibility}
          >
            {#if showBiometricPassword}
              <EyeOff class="h-4 w-4" />
            {:else}
              <Eye class="h-4 w-4" />
            {/if}
          </Button>
        </div>
      </div>
      {#if biometricError}
        <p class="text-destructive text-sm">{biometricError}</p>
      {/if}
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => handleBiometricDialogChange(false)}>Cancel</Button>
      <Button onclick={submitBiometricEnable} disabled={isBiometricActionLoading}>
        {#if isBiometricActionLoading}
          <Spinner class="mr-2 h-4 w-4" />
        {/if}
        Enable
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
