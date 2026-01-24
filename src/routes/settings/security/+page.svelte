<script lang="ts">
  import { get } from 'svelte/store';
  import { onDestroy, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import { securitySettings } from '$lib/stores/security';
  import type { SecuritySettings } from '$lib/config/settings';
  import { totpRequired, totpVerified } from '$lib/stores';
  import { loginTotpSecret, loginTotpConfigured } from '$lib/stores/totp';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { cn } from '$lib/utils';
  import { currentLocale } from '$lib/i18n';
  import {
    Lock,
    RefreshCw,
    Trash2,
    ShieldCheck,
    Fingerprint,
    Smartphone,
    CalendarClock,
    MonitorSmartphone,
    EyeOff,
    Eye,
    TriangleAlert,
    HardDrive,
    Shield,
    Copy,
    Loader2,
    AlertCircle,
    QrCode,
    Check,
    Link2
  } from '@lucide/svelte';
  
  const t = (locale: 'en' | 'sv', en: string, sv: string) => (locale === 'sv' ? sv : en);
  $: locale = $currentLocale as 'en' | 'sv';

  type BooleanSettingKey = {
    [K in keyof SecuritySettings]: SecuritySettings[K] extends boolean ? K : never;
  }[keyof SecuritySettings];

  type AutoLockOption = {
    value: SecuritySettings['autoLockInactivity'];
    label: string;
  };

  type DeviceRecord = {
    id: string;
    name: string;
    kind: string;
    lastSeen: string | null;
    isCurrent: boolean;
  };

  type SecurityActionId = 'rekey' | 'wipe-memory' | 'integrity-check';

  type Argon2Params = {
    memoryKib: number;
    timeCost: number;
    parallelism: number;
  };

  const MIN_PASSWORD_LENGTH = 8;

  const autoLockOptions: AutoLockOption[] = [
    { value: 'Immediate', label: 'Immediate' },
    { value: '1 minute', label: '1 minute' },
    { value: '5 minutes', label: '5 minutes' },
    { value: '15 minutes', label: '15 minutes' },
    { value: 'Custom...', label: 'Custom...' }
  ];

  function getAutoLockLabel(value: SecuritySettings['autoLockInactivity'], locale: 'en' | 'sv'): string {
    if (locale === 'sv') {
      if (value === 'Immediate') return 'Omedelbart';
      if (value === '1 minute') return '1 minut';
      if (value === '5 minutes') return '5 minuter';
      if (value === '15 minutes') return '15 minuter';
      if (value === 'Custom...') return 'Anpassat…';
    }
    return autoLockOptions.find((option) => option.value === value)?.label ?? value;
  }

  const privacyToggles: Array<{
    key: BooleanSettingKey;
    title: string;
    description: string;
  }> = [
    {
      key: 'externalBreachCheck',
      title: 'External Breach Check',
      description: 'Cross-reference vault items against known breach databases.'
    },
    {
      key: 'localReuseDetection',
      title: 'Local Reuse Detection',
      description: 'Alert when passwords repeat across vault entries.'
    },
    {
      key: 'secureRAMHandling',
      title: 'Secure RAM Handling',
      description: 'Allocate hardened memory regions for sensitive operations.'
    }
  ];

  const securityActions: Array<{
    id: SecurityActionId;
    title: string;
    description: string;
    Icon: typeof RefreshCw;
  }> = [
    {
      id: 'rekey',
      title: 'Re-key Vault',
      description: 'Rotate encryption keys and re-encrypt stored data.',
      Icon: RefreshCw
    },
    {
      id: 'wipe-memory',
      title: 'Clear Memory',
      description: 'Scrub sensitive material from memory immediately.',
      Icon: Trash2
    },
    {
      id: 'integrity-check',
      title: 'Integrity Check',
      description: 'Verify vault contents for tampering or corruption.',
      Icon: ShieldCheck
    }
  ];

  const TOTP_CODE_LENGTH = 6;
  const TOTP_ISSUER = 'Pulsar';
  const TOTP_ACCOUNT = 'vault-login';

  let isTotpStatusLoading = true;
  let totpStatusError: string | null = null;
  let pendingTotpSecret: string | null = null;
  let pendingProvisioningUri: string | null = null;
  let totpSetupSuccess: string | null = null;
  let totpDisableSuccess: string | null = null;
  let totpGenerationError: string | null = null;
  let totpVerificationError: string | null = null;
  let totpVerificationCode = '';
  let isGeneratingTotpSecret = false;
  let isConfirmingTotp = false;
  let isDisablingTotp = false;

  type CopyFeedback = {
    context: 'pending' | 'stored';
    message: string;
    variant: 'success' | 'error';
  };

  let secretCopyFeedback: CopyFeedback | null = null;
  let uriCopyFeedback: CopyFeedback | null = null;
  let secretCopyTimeout: ReturnType<typeof setTimeout> | null = null;
  let uriCopyTimeout: ReturnType<typeof setTimeout> | null = null;
  let currentProvisioningUri: string | null = null;

  $: currentProvisioningUri = $loginTotpSecret ? buildProvisioningUri($loginTotpSecret) : null;
  $: generateButtonLabel =
    pendingTotpSecret
      ? t(locale, 'Generate different secret', 'Generera annan hemlighet')
      : $loginTotpConfigured
        ? t(locale, 'Rotate secret', 'Rotera hemlighet')
        : t(locale, 'Generate secret', 'Generera hemlighet');

  let currentSettings: SecuritySettings = get(securitySettings);
  let passwordModalOpen = false;
  let showCurrentPassword = false;
  let currentPassword = '';
  let newPassword = '';
  let confirmPassword = '';
  let changePasswordError = '';
  let isChangingPassword = false;

  let kdfModalOpen = false;
  let kdfCurrentPassword = '';
  let showKdfPassword = false;
  let kdfMemoryMb = 64;
  let kdfTimeCost = 3;
  let kdfParallelism = 4;
  let kdfError = '';
  let isUpdatingKdf = false;

  let argon2Params: Argon2Params = {
    memoryKib: 64 * 1024,
    timeCost: 3,
    parallelism: 4
  };
  let argon2Loading = false;

  let devices: DeviceRecord[] = [];
  let devicesLoading = false;
  let deviceActionPending: Record<string, boolean> = {};
  let isRevokingDevices = false;

  let securityActionPending: Record<SecurityActionId, boolean> = {
    rekey: false,
    'wipe-memory': false,
    'integrity-check': false
  };

  let healthReport: { reusedPasswords: any[], weakPasswordsCount: number } | null = null;
  let healthLoading = false;
  let healthError: string | null = null;

  async function loadSecurityReport() {
    healthLoading = true;
    healthError = null;
    try {
      healthReport = await invoke('get_security_report');
    } catch (error) {
      healthError = parseError(error);
    } finally {
      healthLoading = false;
    }
  }

  // Biometric State
  let isBiometricsEnabled = false;
  let isBiometricActionLoading = false;
  let biometricModalOpen = false;
  let biometricPassword = '';
  let showBiometricPassword = false;
  let biometricError = '';

  const memoryFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
  const gigabyteFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

  const toErrorMessage = (error: unknown): string => {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    return locale === 'sv' ? 'Ett oväntat fel inträffade.' : 'An unexpected error occurred.';
  };

  const formatSecret = (secret: string) => secret.replace(/(.{4})/g, '$1 ').trim();

  // ... (existing functions)

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
      // Disable
      isBiometricActionLoading = true;
      try {
        await invoke('disable_biometrics');
        isBiometricsEnabled = false;
        toast.success('Biometric unlock disabled.');
      } catch (error) {
        toast.error(`Failed to disable biometrics: ${parseError(error)}`);
      } finally {
        isBiometricActionLoading = false;
      }
    } else {
      // Enable - Open Modal
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
        // Authenticate user presence first (native prompt)
        const { authenticate } = await import('@tauri-apps/plugin-biometric');
        await authenticate('Verify identity to enable biometric unlock');
        
        // If successful, store key
        await invoke('enable_biometrics', { password: biometricPassword });
        isBiometricsEnabled = true;
        biometricModalOpen = false;
        toast.success('Biometric unlock enabled.');
    } catch (error) {
        biometricError = parseError(error);
        // If it was a biometric failure, it usually throws.
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
      const expectedToken = await invoke<string>('generate_totp', { secret_b32: pendingTotpSecret });
      if (expectedToken !== totpVerificationCode) {
        totpVerificationError =
          'The verification code did not match. Wait for the next code window and try again.';
        return;
      }

      await invoke('configure_login_totp', { secret_b32: pendingTotpSecret });
      loginTotpSecret.set(pendingTotpSecret);
      loginTotpConfigured.set(true);
      totpSetupSuccess = 'Login TOTP is now enabled. The new secret has been stored on this device.';
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
      await invoke('disable_login_totp');
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
    void refreshTotpStatus();
  });

  const unsubscribe = securitySettings.subscribe((settings) => {
    currentSettings = settings;
  });

  onMount(() => {
    loadArgon2Params();
    loadDevices();
    loadBiometricsStatus();
    loadSecurityReport();
  });

  onDestroy(() => {
    unsubscribe();
    clearCopyTimeouts();
  });

  function applyChanges(partial: Partial<SecuritySettings>) {
    currentSettings = { ...currentSettings, ...partial };
    securitySettings.set(currentSettings);
  }

  function toggleSetting(setting: BooleanSettingKey) {
    applyChanges({ [setting]: !currentSettings[setting] } as Partial<SecuritySettings>);
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

  function getDeviceIcon(kind: string): typeof Fingerprint {
    switch (kind) {
      case 'biometric':
        return Fingerprint;
      case 'device-key':
      case 'key':
        return Smartphone;
      default:
        return Shield;
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
      const params = await invoke<Argon2Params>('get_argon2_params');
      argon2Params = params;
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
      await invoke('rotate_master_password', {
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
      await invoke('update_argon2_params', {
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
      await invoke('wipe_memory');
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
      await invoke('remove_device', { deviceId: device.id });
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
      await invoke('revoke_all_devices');
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

  $: argon2Summary = `Argon2id • memory ${formatArgonMemory(argon2Params.memoryKib)} • time cost ${argon2Params.timeCost} • parallelism ${argon2Params.parallelism}`;

  $: isPasswordFormValid =
    currentPassword.trim().length > 0 &&
    newPassword.trim().length >= MIN_PASSWORD_LENGTH &&
    newPassword.trim() === confirmPassword.trim() &&
    newPassword.trim() !== currentPassword.trim();

  $: isKdfFormValid =
    kdfCurrentPassword.trim().length > 0 &&
    kdfMemoryMb >= 8 &&
    kdfTimeCost >= 1 &&
    kdfParallelism >= 1;
</script>

<div class="flex-1 min-h-0 space-y-6 px-6 py-8">
    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div class="flex w-full items-center justify-between">
        <div>
          <CardTitle>{t(locale, 'Vault Health', 'Valvets hälsa')}</CardTitle>
          <CardDescription>
            {t(locale, 'Security analysis of your stored items.', 'Säkerhetsanalys av dina sparade poster.')}
          </CardDescription>
        </div>
        <Button variant="ghost" size="sm" onclick={loadSecurityReport} disabled={healthLoading}>
          <RefreshCw class={cn("h-4 w-4", healthLoading && "animate-spin")} />
        </Button>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      {#if healthLoading && !healthReport}
        <div class="flex items-center justify-center py-8">
          <Loader2 class="h-8 w-8 animate-spin text-primary/40" />
        </div>
      {:else if healthError}
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{healthError}</AlertDescription>
        </Alert>
      {:else if healthReport}
        {@const reusedCount = healthReport.reusedPasswords.length}
        {@const weakCount = healthReport.weakPasswordsCount}
        
        <div class="grid gap-4 sm:grid-cols-2">
          <div class={cn("rounded-lg border p-4", reusedCount > 0 ? "border-destructive/40 bg-destructive/5" : "border-border/60 bg-muted/20")}>
            <p class="text-sm font-semibold">{t(locale, 'Reused Passwords', 'Återanvända lösenord')}</p>
            <p class={cn("text-2xl font-bold mt-1", reusedCount > 0 ? "text-destructive" : "text-foreground")}>{reusedCount}</p>
            <p class="text-xs text-muted-foreground mt-1">
              {reusedCount > 0 
                ? t(locale, 'Multiple items share the same password.', 'Flera poster delar samma lösenord.')
                : t(locale, 'No password reuse detected.', 'Inga återanvända lösenord upptäckta.')}
            </p>
          </div>

          <div class={cn("rounded-lg border p-4", weakCount > 0 ? "border-warning/40 bg-warning/5" : "border-border/60 bg-muted/20")}>
            <p class="text-sm font-semibold">{t(locale, 'Weak Passwords', 'Svaga lösenord')}</p>
            <p class={cn("text-2xl font-bold mt-1", weakCount > 0 ? "text-warning-foreground" : "text-foreground")}>{weakCount}</p>
            <p class="text-xs text-muted-foreground mt-1">
              {weakCount > 0 
                ? t(locale, 'Passwords shorter than 8 characters.', 'Lösenord kortare än 8 tecken.')
                : t(locale, 'No weak passwords detected.', 'Inga svaga lösenord upptäckta.')}
            </p>
          </div>
        </div>

        {#if reusedCount > 0 || weakCount > 0}
          <div class="mt-4 flex items-start gap-3 rounded-lg border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
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
          <div class="mt-4 flex items-start gap-3 rounded-lg border border-emerald-500/40 bg-emerald-500/10 p-3 text-sm text-emerald-600">
            <Check class="mt-0.5 h-4 w-4 shrink-0" aria-hidden="true" />
            <p>{t(locale, 'Your vault health looks great!', 'Din valv hälsa ser utmärkt ut!')}</p>
          </div>
        {/if}
      {/if}
    </CardContent>
  </Card>

    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Two-factor authentication', 'Tvåfaktorsautentisering')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Protect vault unlocks with a time-based one-time password.', 'Skydda valvupplåsning med tidsbaserade engångskoder.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-5 pt-4">
      {#if totpSetupSuccess}
        <Alert>
          <Check class="h-4 w-4 text-primary" aria-hidden="true" />
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
          <Shield class="h-4 w-4 text-primary" aria-hidden="true" />
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
          <AlertCircle class="h-4 w-4" aria-hidden="true" />
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
          <AlertCircle class="h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>
              {t(locale, 'Unable to generate secret', 'Det gick inte att generera hemlighet')}
            </AlertTitle>
            <AlertDescription>{totpGenerationError}</AlertDescription>
          </div>
        </Alert>
      {/if}

      <div class="space-y-3 rounded-lg border border-border/60 bg-muted/20 p-4">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p class="text-sm font-semibold text-foreground">
              {t(locale, 'Current status', 'Aktuell status')}
            </p>
            <p class="text-sm text-muted-foreground">
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
              $loginTotpConfigured ? 'bg-emerald-500/15 text-emerald-500' : 'bg-muted text-muted-foreground'
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
              <Loader2 class="h-4 w-4 animate-spin" aria-hidden="true" />
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
                <Loader2 class="h-4 w-4 animate-spin" aria-hidden="true" />
              {:else}
                <Shield class="h-4 w-4" aria-hidden="true" />
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
            <RefreshCw class={`h-4 w-4 ${isTotpStatusLoading ? 'animate-spin' : ''}`} aria-hidden="true" />
            Refresh status
          </Button>
        </div>

        {#if isTotpStatusLoading}
          <p class="flex items-center gap-2 text-xs text-muted-foreground">
            <Loader2 class="h-3 w-3 animate-spin" aria-hidden="true" />
            Checking configuration…
          </p>
        {/if}
      </div>

      {#if pendingTotpSecret}
        <div class="space-y-4 rounded-lg border border-primary/40 bg-primary/5 p-4">
          <div class="flex items-center gap-2">
            <QrCode class="h-5 w-5 text-primary" aria-hidden="true" />
            <div>
              <p class="text-sm font-semibold text-foreground">Step 1 — Add the secret to your authenticator</p>
              <p class="text-sm text-muted-foreground">
                Scan the QR code or copy the secret, then confirm a code to finish enabling TOTP.
              </p>
            </div>
          </div>

          <div class="rounded-md border border-border/60 bg-background p-4">
            <p class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Secret</p>
            <p class="mt-2 font-mono text-lg text-foreground break-words">{formatSecret(pendingTotpSecret)}</p>
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
              type="text"
              inputmode="numeric"
              maxlength={TOTP_CODE_LENGTH}
              autocomplete="one-time-code"
              placeholder={t(locale, 'Enter 6-digit code', 'Ange sexsiffrig kod')}
              bind:value={totpVerificationCode}
              oninput={sanitizeTotpCode}
            />
            {#if totpVerificationError}
              <p class="text-sm text-destructive">{totpVerificationError}</p>
            {/if}
          </div>

          {#if totpGenerationError}
            <Alert variant="destructive">
              <AlertCircle class="h-4 w-4" aria-hidden="true" />
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
                <Loader2 class="h-4 w-4 animate-spin" aria-hidden="true" />
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
                <RefreshCw class={`h-4 w-4 ${isGeneratingTotpSecret ? 'animate-spin' : ''}`} aria-hidden="true" />
              {t(locale, 'Generate another secret', 'Generera en ny hemlighet')}
              </Button>
            <Button type="button" variant="ghost" onclick={cancelTotpSetup}>
              {t(locale, 'Cancel', 'Avbryt')}
            </Button>
          </div>

          <p class="text-xs text-muted-foreground">
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
        <div class="space-y-3 rounded-lg border border-border/60 bg-muted/10 p-4">
          <div class="flex items-center gap-2">
            <Shield class="h-5 w-5 text-primary" aria-hidden="true" />
            <div>
              <p class="text-sm font-semibold text-foreground">Stored secret on this device</p>
              <p class="text-sm text-muted-foreground">
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
            <div class="rounded-md border border-border/60 bg-background p-4">
              <p class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Secret</p>
              <p class="mt-2 font-mono text-lg text-foreground break-words">{formatSecret(storedSecret)}</p>
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
              <AlertCircle class="h-4 w-4" aria-hidden="true" />
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

    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Lock class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Master Password & Encryption', 'Huvudlösenord & kryptering')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Manage the master password and key derivation policy.', 'Hantera huvudlösenordet och nyckelderiveringspolicyn.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-5 pt-4">
      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-muted/20 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p class="text-sm font-semibold text-foreground">
            {t(locale, 'Master Password', 'Huvudlösenord')}
          </p>
          <p class="text-sm text-muted-foreground">
            {t(locale, 'Update the password used to unlock your vault.', 'Uppdatera lösenordet som används för att låsa upp ditt valv.')}
          </p>
        </div>
        <Button variant="outline" onclick={openPasswordModal}>
          {t(locale, 'Change Password', 'Ändra lösenord')}
        </Button>
      </div>

      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-muted/20 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p class="text-sm font-semibold text-foreground">
            {t(locale, 'Key Derivation', 'Nyckelderivering')}
          </p>
          <p class="text-sm text-muted-foreground">
            {argon2Loading
              ? t(locale, 'Loading key derivation parameters…', 'Läser in parametrar för nyckelderivering…')
              : argon2Summary}
          </p>
        </div>
        <Button variant="outline" size="sm" onclick={openKdfModal}>
          {t(locale, 'Reconfigure KDF', 'Konfigurera om KDF')}
        </Button>
      </div>
    </CardContent>
  </Card>

    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <CalendarClock class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Auto-lock Controls', 'Kontroller för autolåsning')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Define when the vault should automatically lock itself.', 'Definiera när valvet automatiskt ska låsa sig.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">
            {t(locale, 'Lock on Suspend', 'Lås vid viloläge')}
          </p>
          <p class="text-sm text-muted-foreground">
            {t(locale, 'Lock whenever the system sleeps or hibernates.', 'Lås valvet när systemet går i vila eller viloläge.')}
          </p>
        </div>
        <Switch
          checked={currentSettings.lockOnSuspend}
          aria-label="Toggle lock on suspend"
          onclick={() => toggleSetting('lockOnSuspend')}
        />
      </div>

      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">
            {t(locale, 'Lock on Minimise', 'Lås vid minimering')}
          </p>
          <p class="text-sm text-muted-foreground">
            {t(locale, 'Lock the vault when the window is minimised.', 'Lås valvet när fönstret minimeras.')}
          </p>
        </div>
        <Switch
          checked={currentSettings.lockOnMinimize}
          aria-label="Toggle lock on minimise"
          onclick={() => toggleSetting('lockOnMinimize')}
        />
      </div>

      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-muted/20 px-4 py-4">
        <Label class="text-sm font-semibold text-foreground">
          {t(locale, 'Auto-lock After Inactivity', 'Autolåsning efter inaktivitet')}
        </Label>
        <p class="text-sm text-muted-foreground">
          {t(locale, 'Lock the vault automatically after the selected idle period.', 'Lås valvet automatiskt efter vald inaktivitetsperiod.')}
        </p>
        <Select type="single" value={currentSettings.autoLockInactivity} onValueChange={updateAutoLock}>
          <SelectTrigger aria-label="Select auto-lock inactivity" class="w-full sm:w-56">
            <span data-slot="select-value" class="truncate text-sm">
              {getAutoLockLabel(currentSettings.autoLockInactivity, locale) ||
                t(locale, 'Select duration', 'Välj tidslängd')}
            </span>
          </SelectTrigger>
          <SelectContent>
            {#each autoLockOptions as option}
              <SelectItem value={option.value}>
                {getAutoLockLabel(option.value, locale)}
              </SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>
    </CardContent>
  </Card>

    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Fingerprint class="h-5 w-5" aria-hidden="true" />
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
      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">
            {t(locale, 'Biometric Unlock', 'Biometrisk upplåsning')}
          </p>
          <p class="text-sm text-muted-foreground">
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

      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">
            {t(locale, 'Session Persistence', 'Sessionsbeständighet')}
          </p>
          <p class="text-sm text-muted-foreground">
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

    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
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
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 class="h-4 w-4 animate-spin" aria-hidden="true" />
          <span>{t(locale, 'Loading devices…', 'Läser in enheter…')}</span>
        </div>
      {:else if devices.length === 0}
        <p class="text-sm text-muted-foreground">
          {t(locale, 'No devices have been paired yet.', 'Inga enheter har parkopplats ännu.')}
        </p>
      {:else}
        {#each devices as device}
          <div
            class={cn(
              'flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-4 sm:items-center',
              device.isCurrent ? 'border-primary/60 bg-primary/10' : ''
            )}
          >
            <div class="flex items-start gap-3">
              <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
                <svelte:component this={getDeviceIcon(device.kind)} class="h-5 w-5" aria-hidden="true" />
              </div>
              <div>
                <p class="text-sm font-semibold text-foreground">{device.name}</p>
                <p class="text-xs text-muted-foreground">
                  {(device.lastSeen ??
                    (locale === 'sv' ? 'Ingen senaste aktivitet' : 'No recent activity'))}
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
                  <Loader2 class="h-4 w-4 animate-spin" aria-hidden="true" />
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
              <Loader2 class="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
            {/if}
            {t(locale, 'Revoke All Devices', 'Återkalla alla enheter')}
          </Button>
        </div>
      {/if}
    </CardContent>
  </Card>

    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <EyeOff class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Privacy Controls', 'Integritetskontroller')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Fine-tune privacy and diagnostic data handling.', 'Finjustera integritet och hantering av diagnostikdata.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-5 pt-4">
      <div class="flex flex-col gap-3">
        {#each privacyToggles as toggle}
          <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
            <div>
              <p class="text-sm font-semibold text-foreground">
                {toggle.key === 'externalBreachCheck'
                  ? t(locale, 'External Breach Check', 'Extern intrångskontroll')
                  : toggle.key === 'localReuseDetection'
                    ? t(locale, 'Local Reuse Detection', 'Lokal återanvändningsdetektering')
                    : t(locale, 'Secure RAM Handling', 'Säkrare RAM-hantering')}
              </p>
              <p class="text-sm text-muted-foreground">
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
              checked={currentSettings[toggle.key]}
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

      <div class="rounded-lg border border-border/60 bg-muted/10 p-4">
        <p class="text-sm font-semibold text-foreground">Data Retention Summary</p>
        <ul class="mt-2 list-disc space-y-1 pl-5 text-sm text-muted-foreground">
          <li>Vault data remains encrypted locally and is never uploaded.</li>
          <li>Activity logs rotate every 30 days unless cleared sooner.</li>
          <li>Crash reports are anonymised and retained for up to 90 days.</li>
        </ul>
      </div>
    </CardContent>
  </Card>

    <Card class="border-border/60 bg-card/80 backdrop-blur supports-[backdrop-filter]:bg-card/70">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Shield class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>
          {t(locale, 'Security Actions', 'Säkerhetsåtgärder')}
        </CardTitle>
        <CardDescription>
          {t(locale, 'Execute advanced maintenance and security tasks.', 'Utför avancerade underhålls- och säkerhetsåtgärder.')}
        </CardDescription>
      </div>
    </CardHeader>
    <CardContent class="pt-4">
      <div class="grid gap-3 md:grid-cols-3">
        {#each securityActions as action}
          <Button
            type="button"
            variant="outline"
            class="h-full w-full flex-col items-start gap-3 rounded-xl border-border/60 bg-muted/20 p-4 text-left transition-colors hover:border-primary/50 hover:text-primary"
            onclick={() => handleSecurityAction(action.id)}
            disabled={action.id !== 'rekey' && securityActionPending[action.id]}
            aria-busy={action.id !== 'rekey' && securityActionPending[action.id]}
          >
            {#if action.id !== 'rekey' && securityActionPending[action.id]}
              <Loader2 class="h-5 w-5 animate-spin text-primary" aria-hidden="true" />
            {:else}
              <action.Icon class="h-5 w-5 text-primary" aria-hidden="true" />
            {/if}
            <div class="space-y-1">
              <p class="text-sm font-semibold text-foreground">
                {action.id === 'rekey'
                  ? t(locale, 'Re-key Vault', 'Byt valvnyckel')
                  : action.id === 'wipe-memory'
                    ? t(locale, 'Clear Memory', 'Rensa minne')
                    : t(locale, 'Integrity Check', 'Integritetskontroll')}
              </p>
              <p class="text-xs text-muted-foreground whitespace-normal break-words">
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
            bind:value={currentPassword}
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute right-1 top-1/2 -translate-y-1/2"
            aria-label={showCurrentPassword
              ? t(locale, 'Hide current password', 'Dölj nuvarande lösenord')
              : t(locale, 'Show current password', 'Visa nuvarande lösenord')}
            onclick={togglePasswordVisibility}
          >
            {#if showCurrentPassword}
              <EyeOff class="h-4 w-4 text-primary" aria-hidden="true" />
            {:else}
              <Eye class="h-4 w-4 text-muted-foreground" aria-hidden="true" />
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
          bind:value={newPassword}
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
          bind:value={confirmPassword}
        />
      </div>

      <div class="flex items-start gap-3 rounded-lg border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
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
        <p class="text-sm text-destructive">{changePasswordError}</p>
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
          <Loader2 class="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
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
          <Input id="kdf-memory" type="number" min="8" bind:value={kdfMemoryMb} />
        </div>
        <div class="space-y-2">
          <Label for="kdf-time">{t(locale, 'Time Cost', 'Tidskostnad')}</Label>
          <Input id="kdf-time" type="number" min="1" bind:value={kdfTimeCost} />
        </div>
        <div class="space-y-2">
          <Label for="kdf-parallelism">{t(locale, 'Parallelism', 'Parallellism')}</Label>
          <Input id="kdf-parallelism" type="number" min="1" bind:value={kdfParallelism} />
        </div>
      </div>

      <div class="space-y-2">
        <Label for="kdf-password">{t(locale, 'Current Password', 'Nuvarande lösenord')}</Label>
        <div class="relative">
          <Input
            id="kdf-password"
            type={showKdfPassword ? 'text' : 'password'}
            placeholder={t(locale, 'Enter current password', 'Ange nuvarande lösenord')}
            bind:value={kdfCurrentPassword}
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute right-1 top-1/2 -translate-y-1/2"
            aria-label={showKdfPassword
              ? t(locale, 'Hide current password', 'Dölj nuvarande lösenord')
              : t(locale, 'Show current password', 'Visa nuvarande lösenord')}
            onclick={toggleKdfPasswordVisibility}
          >
            {#if showKdfPassword}
              <EyeOff class="h-4 w-4 text-primary" aria-hidden="true" />
            {:else}
              <Eye class="h-4 w-4 text-muted-foreground" aria-hidden="true" />
            {/if}
          </Button>
        </div>
      </div>

      <div class="flex items-start gap-3 rounded-lg border border-border/60 bg-muted/20 p-3 text-sm text-muted-foreground">
        <TriangleAlert class="mt-0.5 h-4 w-4 text-primary" aria-hidden="true" />
        <p>
          {t(
            locale,
            'Updating Argon2 parameters will re-encrypt the vault and may take a few moments.',
            'Att uppdatera Argon2-parametrarna krypterar om valvet och kan ta en stund.'
          )}
        </p>
      </div>

      {#if kdfError}
        <p class="text-sm text-destructive">{kdfError}</p>
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
          <Loader2 class="mr-2 h-4 w-4 animate-spin" aria-hidden="true" />
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
            bind:value={biometricPassword}
            onkeydown={(e) => e.key === 'Enter' && submitBiometricEnable()}
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute right-1 top-1/2 -translate-y-1/2"
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
        <p class="text-sm text-destructive">{biometricError}</p>
      {/if}
    </div>
    <DialogFooter>
      <Button variant="outline" onclick={() => handleBiometricDialogChange(false)}>Cancel</Button>
      <Button onclick={submitBiometricEnable} disabled={isBiometricActionLoading}>
        {#if isBiometricActionLoading}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        Enable
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
