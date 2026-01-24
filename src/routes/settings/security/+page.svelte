<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { totpVerified, totpRequired } from '$lib/stores';
  import { securitySettings } from '$lib/stores/security';
  import { loginTotpSecret, loginTotpConfigured } from '$lib/stores/totp';
  import { callBackend } from '$lib/utils/backend';
  import { invoke } from '@tauri-apps/api/core';
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
  import Input from '$lib/components/ui/Input.svelte';
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
    Smartphone,
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
    MonitorSmartphone,
    HardDrive
  } from '@lucide/svelte';
  import { currentLocale } from '$lib/i18n';
  import type { SecuritySettings } from '$lib/config/settings';
  import { cn } from '$lib/utils';
  import { toast } from 'svelte-sonner';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';

  interface Argon2Params {
    memoryKib: number;
    timeCost: number;
    parallelism: number;
  }

  interface DeviceRecord {
    id: string;
    name: string;
    kind: string;
    lastSeen: string | null;
    isCurrent: boolean;
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

  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  let locale = $derived($currentLocale as 'en' | 'sv');

  let currentSettings = $state<SecuritySettings>({ ...get(securitySettings) });

  const unsubscribe = securitySettings.subscribe((settings) => {
    currentSettings = { ...settings };
  });

  const currentProvisioningUri = $derived(
    $loginTotpSecret ? buildProvisioningUri($loginTotpSecret) : null
  );
  const generateButtonLabel = $derived(
    $loginTotpSecret
      ? t(locale, 'Regenerate Secret', 'Regenerera hemlighet')
      : t(locale, 'Generate Secret', 'Generera hemlighet')
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
  let kdfMemoryMb = $state(64);
  let kdfTimeCost = $state(3);
  let kdfParallelism = $state(4);
  let kdfError = $state('');
  let isUpdatingKdf = $state(false);

  let argon2Params = $state<Argon2Params>({
    memoryKib: 64 * 1024,
    timeCost: 3,
    parallelism: 4
  });
  let argon2Loading = $state(false);

  let devices = $state<DeviceRecord[]>([]);
  let devicesLoading = $state(false);
  let deviceActionPending = $state<Record<string, boolean>>({});
  let isRevokingDevices = $state(false);

  let securityActionPending = $state<Record<SecurityActionId, boolean>>({
    rekey: false,
    'wipe-memory': false,
    'integrity-check': false
  });

  let healthReport = $state<{ reusedPasswords: any[]; weakPasswordsCount: number } | null>(null);
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

  const toErrorMessage = (error: unknown): string => {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    return locale === 'sv' ? 'Ett oväntat fel inträffade.' : 'An unexpected error occurred.';
  };

  const formatSecret = (secret: string) => secret.replace(/(.{4})/g, '$1 ').trim();

  async function loadSecurityReport() {
    healthLoading = true;
    healthError = null;
    try {
      healthReport = await callBackend('get_security_report');
    } catch (error) {
      healthError = parseError(error);
    } finally {
      healthLoading = false;
    }
  }

  async function loadBiometricsStatus() {
    try {
      isBiometricsEnabled = await invoke<boolean>('is_biometrics_enabled');
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
      const configured = await invoke<boolean>('is_login_totp_configured');
      loginTotpConfigured.set(configured);
      if (!configured) {
        loginTotpSecret.set(null);
      }
    } catch (error) {
      totpStatusError = toErrorMessage(error);
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
      const secret = await invoke<string>('generate_totp_secret');
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
      totpGenerationError = toErrorMessage(error);
    } finally {
      isGeneratingTotpSecret = false;
    }
  }

  async function copySecret(secret: string | null, context: CopyFeedback['context']) {
    if (!secret) return;

    try {
      await writeText(secret);
      setSecretCopyFeedback(context, 'Secret copied to clipboard.', 'success');
    } catch (error) {
      setSecretCopyFeedback(context, toErrorMessage(error), 'error');
    }
  }

  async function copyProvisioningUri(uri: string | null, context: CopyFeedback['context']) {
    if (!uri) return;

    try {
      await writeText(uri);
      setUriCopyFeedback(context, 'Setup link copied to clipboard.', 'success');
    } catch (error) {
      setUriCopyFeedback(context, toErrorMessage(error), 'error');
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
      const expectedToken = await invoke<string>('generate_totp', {
        secret_b32: pendingTotpSecret
      });
      if (expectedToken !== totpVerificationCode) {
        totpVerificationError =
          'The verification code did not match. Wait for the next code window and try again.';
        return;
      }

      await callBackend('configure_login_totp', { secret_b32: pendingTotpSecret });
      loginTotpSecret.set(pendingTotpSecret);
      loginTotpConfigured.set(true);
      totpSetupSuccess =
        'Login TOTP is now enabled. The new secret has been stored on this device.';
      totpDisableSuccess = null;
      pendingTotpSecret = null;
      pendingProvisioningUri = null;
      totpVerificationCode = '';
      totpRequired.set(false);
      totpVerified.set(true);
      await refreshTotpStatus();
    } catch (error) {
      totpVerificationError = toErrorMessage(error);
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
      loginTotpConfigured.set(false);
      loginTotpSecret.set(null);
      pendingTotpSecret = null;
      pendingProvisioningUri = null;
      totpVerificationCode = '';
      totpDisableSuccess = 'Login TOTP disabled. Unlocking will only require your master password.';
      totpRequired.set(false);
      totpVerified.set(false);
      await refreshTotpStatus();
    } catch (error) {
      totpStatusError = toErrorMessage(error);
    } finally {
      isDisablingTotp = false;
    }
  }

  onMount(() => {
    loadArgon2Params();
    loadDevices();
    loadBiometricsStatus();
    loadSecurityReport();
    refreshTotpStatus();
  });

  onDestroy(() => {
    unsubscribe();
    clearCopyTimeouts();
  });

  function applyChanges(partial: Partial<SecuritySettings>) {
    currentSettings = { ...currentSettings, ...partial };
    securitySettings.set(currentSettings);
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

  function parseError(error: unknown): string {
    if (error instanceof Error) {
      return error.message;
    }
    if (typeof error === 'string') {
      return error;
    }
    try {
      return JSON.stringify(error);
    } catch {
      return 'Unknown error';
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
    kdfMemoryMb = Math.max(8, Math.round(argon2Params.memoryKib / 1024));
    kdfTimeCost = argon2Params.timeCost;
    kdfParallelism = argon2Params.parallelism;
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

  function getDeviceIcon(kind: string): any {
    switch (kind) {
      case 'biometric':
        return FingerprintPattern;
      case 'device-key':
      case 'key':
        return Smartphone;
      default:
        return ShieldCheck;
    }
  }

  function getDeviceTypeLabel(kind: string): string {
    if (!kind) {
      return 'Unknown';
    }
    return kind
      .split(/[-_ ]+/)
      .filter(Boolean)
      .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
      .join(' ');
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

  async function loadDevices() {
    devicesLoading = true;
    try {
      const result = await invoke<DeviceRecord[]>('list_devices');
      devices = result.map((device) => ({
        ...device,
        kind: device.kind ?? 'unknown',
        lastSeen: device.lastSeen ?? null
      }));
      deviceActionPending = {};
    } catch (error) {
      toast.error(`Failed to load devices: ${parseError(error)}`);
    } finally {
      devicesLoading = false;
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
      if (kdfCurrentPassword.trim().length === 0) {
        kdfError = 'Current password is required.';
      } else if (kdfMemoryMb < 8) {
        kdfError = 'Memory must be at least 8 MiB.';
      } else if (kdfTimeCost < 1) {
        kdfError = 'Time cost must be at least 1.';
      } else if (kdfParallelism < 1) {
        kdfError = 'Parallelism must be at least 1.';
      }
      return;
    }

    isUpdatingKdf = true;
    try {
      await callBackend('update_argon2_params', {
        currentPassword: kdfCurrentPassword,
        memoryKib: Math.round(kdfMemoryMb * 1024),
        timeCost: Math.round(kdfTimeCost),
        parallelism: Math.round(kdfParallelism)
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
      const result = await invoke<string>('run_integrity_check');
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

  async function removeDevice(device: DeviceRecord) {
    deviceActionPending = { ...deviceActionPending, [device.id]: true };
    try {
      await callBackend('remove_device', { deviceId: device.id });
      devices = devices.filter((entry) => entry.id !== device.id);
      const updated = { ...deviceActionPending };
      delete updated[device.id];
      deviceActionPending = updated;
      toast.success(`Removed ${device.name}.`);
    } catch (error) {
      deviceActionPending = { ...deviceActionPending, [device.id]: false };
      toast.error(`Failed to remove device: ${parseError(error)}`);
    }
  }

  async function revokeAllDevices() {
    isRevokingDevices = true;
    try {
      await callBackend('revoke_all_devices');
      devices = [];
      deviceActionPending = {};
      toast.success('All devices revoked.');
    } catch (error) {
      toast.error(`Failed to revoke devices: ${parseError(error)}`);
    } finally {
      isRevokingDevices = false;
    }
  }

  function handlePairDevice() {
    toast.info('Device pairing is not yet available. Stay tuned!');
  }

  const argon2Summary = $derived(
    `Argon2id • memory ${formatArgonMemory(argon2Params.memoryKib)} • time cost ${argon2Params.timeCost} • parallelism ${argon2Params.parallelism}`
  );

  const isPasswordFormValid = $derived(
    currentPassword.length > 0 && newPassword.length >= 8 && newPassword === confirmPassword
  );

  const isKdfFormValid = $derived(
    kdfCurrentPassword.length > 0 && kdfMemoryMb >= 8 && kdfTimeCost >= 1 && kdfParallelism >= 1
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
          <CardTitle>{t(locale, 'Vault Health', 'Valvets hälsa')}</CardTitle>
          <CardDescription>
            {t(
              locale,
              'Security analysis of your stored items.',
              'Säkerhetsanalys av dina sparade poster.'
            )}
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
        {@const weakCount = healthReport.weakPasswordsCount}

        <div class="grid gap-4 sm:grid-cols-2">
          <div
            class={cn(
              'rounded-lg border p-4',
              reusedCount > 0
                ? 'border-destructive/40 bg-destructive/5'
                : 'border-border/60 bg-muted/20'
            )}
          >
            <p class="text-sm font-semibold">
              {t(locale, 'Reused Passwords', 'Återanvända lösenord')}
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
                ? t(
                    locale,
                    'Multiple items share the same password.',
                    'Flera poster delar samma lösenord.'
                  )
                : t(locale, 'No password reuse detected.', 'Inga återanvända lösenord upptäckta.')}
            </p>
          </div>

          <div
            class={cn(
              'rounded-lg border p-4',
              weakCount > 0 ? 'border-warning/40 bg-warning/5' : 'border-border/60 bg-muted/20'
            )}
          >
            <p class="text-sm font-semibold">{t(locale, 'Weak Passwords', 'Svaga lösenord')}</p>
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
                ? t(locale, 'Passwords shorter than 8 characters.', 'Lösenord kortare än 8 tecken.')
                : t(locale, 'No weak passwords detected.', 'Inga svaga lösenord upptäckta.')}
            </p>
          </div>
        </div>

        {#if reusedCount > 0 || weakCount > 0}
          <div
            class="border-destructive/40 bg-destructive/10 text-destructive mt-4 flex items-start gap-3 rounded-lg border p-3 text-sm"
          >
            <TriangleAlert class="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
            <p>
              {t(
                locale,
                'Security issues detected. Consider updating shared or short passwords to improve vault integrity.',
                'Säkerhetsproblem upptäckta. Överväg att uppdatera delade eller korta lösenord för att förbättra valvets säkerhet.'
              )}
            </p>
          </div>
        {:else}
          <div
            class="mt-4 flex items-start gap-3 rounded-lg border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm text-emerald-600"
          >
            <Check class="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
            <p>{t(locale, 'Your vault health looks great!', 'Din valv hälsa ser utmärkt ut!')}</p>
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
          {t(locale, 'Two-factor authentication', 'Tvåfaktorsautentisering')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Protect vault unlocks with a time-based one-time password.',
            'Skydda valvupplåsning med tidsbaserade engångskoder.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-5 pt-4">
      {#if totpSetupSuccess}
        <Alert>
          <Check class="text-primary h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>
              {t(locale, 'Authenticator enabled', 'Autentiserare aktiverad')}
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
              {t(locale, 'Authenticator disabled', 'Autentiserare inaktiverad')}
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
              {t(locale, 'Unable to load status', 'Det gick inte att läsa in status')}
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
              {t(locale, 'Unable to generate secret', 'Det gick inte att generera hemlighet')}
            </AlertTitle>
            <AlertDescription>{totpGenerationError}</AlertDescription>
          </div>
        </Alert>
      {/if}

      <div class="border-border/60 bg-muted/20 space-y-3 rounded-lg border p-4">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p class="text-foreground text-sm font-semibold">
              {t(locale, 'Current status', 'Aktuell status')}
            </p>
            <p class="text-muted-foreground text-sm">
              {$loginTotpConfigured
                ? t(
                    locale,
                    'Unlocking requires both your master password and an authenticator token.',
                    'Upplåsning kräver både ditt huvudlösenord och en autentiseringskod.'
                  )
                : t(
                    locale,
                    'Generate a secret to require an authenticator token when unlocking the vault.',
                    'Generera en hemlighet för att kräva en autentiseringskod vid upplåsning.'
                  )}
            </p>
          </div>
          <Badge
            class={cn(
              'px-3 py-1 text-xs font-medium',
              $loginTotpConfigured
                ? 'bg-emerald-500/15 text-emerald-500'
                : 'bg-muted text-muted-foreground'
            )}
          >
            {$loginTotpConfigured ? 'Enabled' : 'Disabled'}
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

          {#if $loginTotpConfigured}
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
              {t(locale, 'Step 2 — Confirm a code', 'Steg 2 — Bekräfta en kod')}
            </Label>
            <Input
              id="totp-verification"
              type="number"
              maxlength={TOTP_CODE_LENGTH}
              autocomplete="one-time-code"
              placeholder={t(locale, 'Enter 6-digit code', 'Ange sexsiffrig kod')}
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
                  {t(locale, 'Unable to generate secret', 'Det gick inte att generera hemlighet')}
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
              {isConfirmingTotp
                ? t(locale, 'Verifying…', 'Verifierar…')
                : t(locale, 'Verify & enable', 'Verifiera och aktivera')}
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
              {t(locale, 'Generate another secret', 'Generera en ny hemlighet')}
            </Button>
            <Button type="button" variant="ghost" onclick={cancelTotpSetup}>
              {t(locale, 'Cancel', 'Avbryt')}
            </Button>
          </div>

          <p class="text-muted-foreground text-xs">
            {t(
              locale,
              'Codes rotate every 30 seconds. If verification fails, wait for the next code before trying again.',
              'Koder roterar var 30:e sekund. Om verifiering misslyckas, vänta på nästa kod innan du försöker igen.'
            )}
          </p>
        </div>
      {/if}

      {#if $loginTotpConfigured && !pendingTotpSecret}
        {@const storedSecret = $loginTotpSecret}
        <div class="border-border/60 bg-muted/10 space-y-3 rounded-lg border p-4">
          <div class="flex items-center gap-2">
            <ShieldCheck class="text-primary h-5 w-5" aria-hidden="true" />
            <div>
              <p class="text-foreground text-sm font-semibold">Stored secret on this device</p>
              <p class="text-muted-foreground text-sm">
                {storedSecret
                  ? t(
                      locale,
                      'Copy the secret if you need to enrol another authenticator or keep an offline backup.',
                      'Kopiera hemligheten om du behöver registrera en annan autentiserare eller spara en offline-kopia.'
                    )
                  : t(
                      locale,
                      'This device does not have a local copy of the secret. Rotate the secret to capture it again.',
                      'Denna enhet har ingen lokal kopia av hemligheten. Rotera hemligheten för att fånga den igen.'
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
                  {t(locale, 'Copy secret', 'Kopiera hemlighet')}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  class="gap-2"
                  onclick={() => copyProvisioningUri(currentProvisioningUri, 'stored')}
                  disabled={!currentProvisioningUri}
                >
                  <Link2 class="h-4 w-4" aria-hidden="true" />
                  {t(locale, 'Copy setup link', 'Kopiera installationslänk')}
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
                  {t(locale, 'No local secret available', 'Ingen lokal hemlighet tillgänglig')}
                </AlertTitle>
                <AlertDescription>
                  {t(
                    locale,
                    'Rotate the secret to store a copy on this device for backup access.',
                    'Rotera hemligheten för att spara en kopia på denna enhet för backupåtkomst.'
                  )}
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
          {t(locale, 'Master Password & Encryption', 'Huvudlösenord & kryptering')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Manage the master password and key derivation policy.',
            'Hantera huvudlösenordet och nyckelderiveringspolicyn.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-5 pt-4">
      <div
        class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Master Password', 'Huvudlösenord')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Update the password used to unlock your vault.',
              'Uppdatera lösenordet som används för att låsa upp ditt valv.'
            )}
          </p>
        </div>
        <Button variant="outline" onclick={openPasswordModal}>
          {t(locale, 'Change Password', 'Ändra lösenord')}
        </Button>
      </div>

      <div
        class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Key Derivation', 'Nyckelderivering')}
          </p>
          <p class="text-muted-foreground text-sm">
            {argon2Loading
              ? t(
                  locale,
                  'Loading key derivation parameters…',
                  'Läser in parametrar för nyckelderivering…'
                )
              : argon2Summary}
          </p>
        </div>
        <Button variant="outline" size="sm" onclick={openKdfModal}>
          {t(locale, 'Reconfigure KDF', 'Konfigurera om KDF')}
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
          {t(locale, 'Auto-lock Controls', 'Kontroller för autolåsning')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Define when the vault should automatically lock itself.',
            'Definiera när valvet automatiskt ska låsa sig.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Lock on Suspend', 'Lås vid viloläge')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Lock whenever the system sleeps or hibernates.',
              'Lås valvet när systemet går i vila eller viloläge.'
            )}
          </p>
        </div>
        <Switch
          checked={currentSettings.lockOnSuspend}
          aria-label="Toggle lock on suspend"
          onclick={() => toggleSetting('lockOnSuspend')}
        />
      </div>

      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Lock on Minimise', 'Lås vid minimering')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Lock the vault when the window is minimised.',
              'Lås valvet när fönstret minimeras.'
            )}
          </p>
        </div>
        <Switch
          checked={currentSettings.lockOnMinimize}
          aria-label="Toggle lock on minimise"
          onclick={() => toggleSetting('lockOnMinimize')}
        />
      </div>

      <div class="border-border/60 bg-muted/20 flex flex-col gap-2 rounded-lg border px-4 py-4">
        <Label class="text-foreground text-sm font-semibold">
          {t(locale, 'Auto-lock After Inactivity', 'Autolåsning efter inaktivitet')}
        </Label>
        <p class="text-muted-foreground text-sm">
          {t(
            locale,
            'Lock the vault automatically after the selected idle period.',
            'Lås valvet automatiskt efter vald inaktivitetsperiod.'
          )}
        </p>
        <Select
          type="single"
          value={currentSettings.autoLockInactivity}
          onValueChange={updateAutoLock}
        >
          <SelectTrigger aria-label="Select auto-lock inactivity" class="w-full sm:w-56">
            <span data-slot="select-value" class="truncate text-sm">
              {getAutoLockLabel(currentSettings.autoLockInactivity) ||
                t(locale, 'Select duration', 'Välj tidslängd')}
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
          {t(locale, 'Biometric & Session', 'Biometrik & session')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Control biometric unlock availability and session persistence.',
            'Styr när biometrisk upplåsning är tillgänglig och hur sessioner bevaras.'
          )}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Biometric Unlock', 'Biometrisk upplåsning')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Allow fingerprint or face recognition to unlock the vault.',
              'Tillåt fingeravtryck eller ansiktsigenkänning för att låsa upp valvet.'
            )}
          </p>
        </div>
        <Switch
          checked={isBiometricsEnabled}
          disabled={isBiometricActionLoading}
          aria-label="Toggle biometric unlock"
          onclick={handleBiometricToggle}
        />
      </div>

      <div
        class="border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-3"
      >
        <div>
          <p class="text-foreground text-sm font-semibold">
            {t(locale, 'Session Persistence', 'Sessionsbeständighet')}
          </p>
          <p class="text-muted-foreground text-sm">
            {t(
              locale,
              'Remember the unlocked session between restarts.',
              'Kom ihåg upplåst session mellan omstarter.'
            )}
          </p>
        </div>
        <Switch
          checked={currentSettings.sessionPersistence}
          aria-label="Toggle session persistence"
          onclick={() => toggleSetting('sessionPersistence')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-card/80 supports-backdrop-filter:bg-card/70 backdrop-blur">
    <CardHeader class="border-border/40 flex flex-row items-start gap-3 border-b pb-4">
      <div
        class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
      >
        <MonitorSmartphone class="h-5 w-5" aria-hidden="true" />
      </div>
      <div class="flex w-full flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <CardTitle>
            {t(locale, 'Paired Devices', 'Parkopplade enheter')}
          </CardTitle>
          <CardDescription>
            {t(
              locale,
              'Review devices authorised for biometric or key-based unlock.',
              'Granska enheter som är auktoriserade för biometrisk eller nyckelbaserad upplåsning.'
            )}
          </CardDescription>
        </div>
        <Button variant="outline" size="sm" onclick={handlePairDevice} disabled={devicesLoading}>
          {t(locale, 'Pair New Device', 'Parkoppla ny enhet')}
        </Button>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      {#if devicesLoading}
        <div class="text-muted-foreground flex items-center gap-2 text-sm">
          <Spinner class="h-4 w-4" aria-hidden="true" />
          <span>{t(locale, 'Loading devices…', 'Läser in enheter…')}</span>
        </div>
      {:else if devices.length === 0}
        <p class="text-muted-foreground text-sm">
          {t(locale, 'No devices have been paired yet.', 'Inga enheter har parkopplats ännu.')}
        </p>
      {:else}
        {#each devices as device (device.id)}
          {@const DeviceIcon = getDeviceIcon(device.kind)}
          <div
            class={cn(
              'border-border/60 bg-muted/20 flex items-start justify-between gap-4 rounded-lg border px-4 py-4 sm:items-center',
              device.isCurrent ? 'border-primary/60 bg-primary/10' : ''
            )}
          >
            <div class="flex items-start gap-3">
              <div
                class="bg-primary/10 text-primary flex h-10 w-10 items-center justify-center rounded-full"
              >
                <DeviceIcon class="h-5 w-5" aria-hidden="true" />
              </div>
              <div>
                <p class="text-foreground text-sm font-semibold">{device.name}</p>
                <p class="text-muted-foreground text-xs">
                  {device.lastSeen ??
                    (locale === 'sv' ? 'Ingen senaste aktivitet' : 'No recent activity')}
                  {device.isCurrent
                    ? locale === 'sv'
                      ? ' • Aktuell enhet'
                      : ' • Current device'
                    : ''}
                </p>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <Badge variant="secondary">{getDeviceTypeLabel(device.kind)}</Badge>
              <Button
                variant="ghost"
                size="icon"
                onclick={() => removeDevice(device)}
                disabled={!!deviceActionPending[device.id]}
                aria-label={`Remove ${device.name}`}
              >
                {#if deviceActionPending[device.id]}
                  <Spinner class="h-4 w-4" aria-hidden="true" />
                {:else}
                  <Trash2 class="h-4 w-4" aria-hidden="true" />
                {/if}
              </Button>
            </div>
          </div>
        {/each}

        <div class="flex justify-end">
          <Button
            variant="destructive"
            size="sm"
            onclick={revokeAllDevices}
            disabled={isRevokingDevices || devices.length === 0}
          >
            {#if isRevokingDevices}
              <Spinner class="mr-2 h-4 w-4" aria-hidden="true" />
            {/if}
            {t(locale, 'Revoke All Devices', 'Återkalla alla enheter')}
          </Button>
        </div>
      {/if}
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
          {t(locale, 'Privacy Controls', 'Integritetskontroller')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Fine-tune privacy and diagnostic data handling.',
            'Finjustera integritet och hantering av diagnostikdata.'
          )}
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
                  ? t(locale, 'External Breach Check', 'Extern intrångskontroll')
                  : toggle.key === 'localReuseDetection'
                    ? t(locale, 'Local Reuse Detection', 'Lokal återanvändningsdetektering')
                    : t(locale, 'Secure RAM Handling', 'Säkrare RAM-hantering')}
              </p>
              <p class="text-muted-foreground text-sm">
                {toggle.key === 'externalBreachCheck'
                  ? t(
                      locale,
                      'Cross-reference vault items against known breach databases.',
                      'Jämför valvposter mot kända läckage-databaser.'
                    )
                  : toggle.key === 'localReuseDetection'
                    ? t(
                        locale,
                        'Alert when passwords repeat across vault entries.',
                        'Varna när lösenord återanvänds mellan poster i valvet.'
                      )
                    : t(
                        locale,
                        'Allocate hardened memory regions for sensitive operations.',
                        'Tilldela härdade minnesområden för känsliga operationer.'
                      )}
              </p>
            </div>
            <Switch
              checked={currentSettings[toggle.key] as boolean}
              aria-label={`Toggle ${toggle.title}`}
              onclick={() => toggleSetting(toggle.key)}
            />
          </div>
        {/each}
      </div>

      <div class="grid gap-3 sm:grid-cols-2">
        <Button variant="outline" class="justify-start gap-3" onclick={() => {}}>
          <HardDrive class="h-4 w-4" aria-hidden="true" />
          {t(locale, 'Access Local Logs', 'Öppna lokala loggar')}
        </Button>
        <Button variant="outline" class="justify-start gap-3" onclick={() => {}}>
          <Trash2 class="h-4 w-4" aria-hidden="true" />
          {t(locale, 'Clear Local Logs', 'Rensa lokala loggar')}
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
          {t(locale, 'Security Actions', 'Säkerhetsåtgärder')}
        </CardTitle>
        <CardDescription>
          {t(
            locale,
            'Execute advanced maintenance and security tasks.',
            'Utför avancerade underhålls- och säkerhetsåtgärder.'
          )}
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
                  ? t(locale, 'Re-key Vault', 'Byt valvnyckel')
                  : action.id === 'wipe-memory'
                    ? t(locale, 'Clear Memory', 'Rensa minne')
                    : t(locale, 'Integrity Check', 'Integritetskontroll')}
              </p>
              <p class="text-muted-foreground text-xs wrap-break-word whitespace-normal">
                {action.id === 'rekey'
                  ? t(
                      locale,
                      'Rotate encryption keys and re-encrypt stored data.',
                      'Rotera krypteringsnycklar och kryptera om lagrad data.'
                    )
                  : action.id === 'wipe-memory'
                    ? t(
                        locale,
                        'Scrub sensitive material from memory immediately.',
                        'Rensa känslig information från minnet omedelbart.'
                      )
                    : t(
                        locale,
                        'Verify vault contents for tampering or corruption.',
                        'Verifiera valvets innehåll för manipulation eller korruption.'
                      )}
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
        {t(locale, 'Change Master Password', 'Byt huvudlösenord')}
      </DialogTitle>
      <DialogDescription>
        {t(
          locale,
          'Provide your current master password and enter a new secure password to re-encrypt the vault.',
          'Ange ditt nuvarande huvudlösenord och ett nytt säkert lösenord för att kryptera om valvet.'
        )}
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="current-password">
          {t(locale, 'Current Password', 'Nuvarande lösenord')}
        </Label>
        <div class="relative">
          <Input
            id="current-password"
            type={showCurrentPassword ? 'text' : 'password'}
            placeholder={t(locale, 'Enter current password', 'Ange nuvarande lösenord')}
            bind:inputValue={currentPassword}
            title="Current Password"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute top-1/2 right-1 -translate-y-1/2"
            aria-label={showCurrentPassword
              ? t(locale, 'Hide current password', 'Dölj nuvarande lösenord')
              : t(locale, 'Show current password', 'Visa nuvarande lösenord')}
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
        <Label for="new-password">{t(locale, 'New Password', 'Nytt lösenord')}</Label>
        <Input
          id="new-password"
          type="password"
          placeholder={t(locale, 'Enter new password', 'Ange nytt lösenord')}
          bind:inputValue={newPassword}
          title="New Password"
        />
      </div>

      <div class="space-y-2">
        <Label for="confirm-password">
          {t(locale, 'Confirm New Password', 'Bekräfta nytt lösenord')}
        </Label>
        <Input
          id="confirm-password"
          type="password"
          placeholder={t(locale, 'Confirm new password', 'Bekräfta nytt lösenord')}
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
            locale,
            'Changing the master password re-encrypts the vault. The operation may take several minutes for large vaults.',
            'Att byta huvudlösenord krypterar om valvet och kan ta några minuter för stora valv.'
          )}
        </p>
      </div>
      {#if changePasswordError}
        <p class="text-destructive text-sm">{changePasswordError}</p>
      {/if}
    </div>

    <DialogFooter class="gap-2">
      <Button type="button" variant="outline" onclick={closePasswordModal}>
        {t(locale, 'Cancel', 'Avbryt')}
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
        {t(locale, 'Change Password', 'Byt lösenord')}
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog open={kdfModalOpen} onOpenChange={handleKdfDialogChange}>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>
        {t(locale, 'Reconfigure Key Derivation', 'Ändra nyckelderivering')}
      </DialogTitle>
      <DialogDescription>
        {t(
          locale,
          'Adjust the Argon2id parameters used when deriving the vault encryption key.',
          'Justera Argon2id-parametrarna som används för att härleda valvets krypteringsnyckel.'
        )}
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      <div class="grid gap-3 sm:grid-cols-3">
        <div class="space-y-2">
          <Label for="kdf-memory">{t(locale, 'Memory (MiB)', 'Minne (MiB)')}</Label>
          <Input
            id="kdf-memory"
            type="number"
            min="8"
            bind:inputValue={kdfMemoryMb as any}
            title="Memory"
          />
        </div>
        <div class="space-y-2">
          <Label for="kdf-time">{t(locale, 'Time Cost', 'Tidskostnad')}</Label>
          <Input
            id="kdf-time"
            type="number"
            min="1"
            bind:inputValue={kdfTimeCost as any}
            title="Time Cost"
          />
        </div>
        <div class="space-y-2">
          <Label for="kdf-parallelism">{t(locale, 'Parallelism', 'Parallellism')}</Label>
          <Input
            id="kdf-parallelism"
            type="number"
            min="1"
            bind:inputValue={kdfParallelism as any}
            title="Parallelism"
          />
        </div>
      </div>

      <div class="space-y-2">
        <Label for="kdf-password">{t(locale, 'Current Password', 'Nuvarande lösenord')}</Label>
        <div class="relative">
          <Input
            id="kdf-password"
            type={showKdfPassword ? 'text' : 'password'}
            placeholder={t(locale, 'Enter current password', 'Ange nuvarande lösenord')}
            bind:inputValue={kdfCurrentPassword}
            title="Password"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute top-1/2 right-1 -translate-y-1/2"
            aria-label={showKdfPassword
              ? t(locale, 'Hide current password', 'Dölj nuvarande lösenord')
              : t(locale, 'Show current password', 'Visa nuvarande lösenord')}
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
          {t(
            locale,
            'Updating Argon2 parameters will re-encrypt the vault and may take a few moments.',
            'Att uppdatera Argon2-parametrarna krypterar om valvet och kan ta en stund.'
          )}
        </p>
      </div>

      {#if kdfError}
        <p class="text-destructive text-sm">{kdfError}</p>
      {/if}
    </div>

    <DialogFooter class="gap-2">
      <Button type="button" variant="outline" onclick={() => handleKdfDialogChange(false)}>
        {t(locale, 'Cancel', 'Avbryt')}
      </Button>
      <Button
        type="button"
        onclick={submitKdfUpdate}
        disabled={!isKdfFormValid || isUpdatingKdf}
        aria-busy={isUpdatingKdf}
      >
        {#if isUpdatingKdf}
          <Spinner class="mr-2 h-4 w-4" aria-hidden="true" />
        {/if}
        {t(locale, 'Apply Changes', 'Verkställ ändringar')}
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
