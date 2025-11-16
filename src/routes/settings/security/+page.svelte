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
  $: generateButtonLabel = pendingTotpSecret
    ? 'Generate different secret'
    : $loginTotpConfigured
      ? 'Rotate secret'
      : 'Generate secret';

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

  const memoryFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });
  const gigabyteFormatter = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });

  const toErrorMessage = (error: unknown): string => {
    if (typeof error === 'string') return error;
    if (error instanceof Error) return error.message;
    return 'An unexpected error occurred.';
  };

  const formatSecret = (secret: string) => secret.replace(/(.{4})/g, '$1 ').trim();

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

<div class="flex min-h-0 flex-1 flex-col gap-6 px-8 py-8">
  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <ShieldCheck class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Two-factor authentication</CardTitle>
        <CardDescription>Protect vault unlocks with a time-based one-time password.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="space-y-5 pt-4">
      {#if totpSetupSuccess}
        <Alert>
          <Check class="h-4 w-4 text-primary" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>Authenticator enabled</AlertTitle>
            <AlertDescription>{totpSetupSuccess}</AlertDescription>
          </div>
        </Alert>
      {/if}

      {#if totpDisableSuccess}
        <Alert>
          <Shield class="h-4 w-4 text-primary" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>Authenticator disabled</AlertTitle>
            <AlertDescription>{totpDisableSuccess}</AlertDescription>
          </div>
        </Alert>
      {/if}

      {#if totpStatusError}
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>Unable to load status</AlertTitle>
            <AlertDescription>{totpStatusError}</AlertDescription>
          </div>
        </Alert>
      {/if}

      {#if totpGenerationError && !pendingTotpSecret}
        <Alert variant="destructive">
          <AlertCircle class="h-4 w-4" aria-hidden="true" />
          <div class="space-y-1">
            <AlertTitle>Unable to generate secret</AlertTitle>
            <AlertDescription>{totpGenerationError}</AlertDescription>
          </div>
        </Alert>
      {/if}

      <div class="space-y-3 rounded-lg border border-border/60 bg-muted/20 p-4">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p class="text-sm font-semibold text-foreground">Current status</p>
            <p class="text-sm text-muted-foreground">
              {$loginTotpConfigured
                ? 'Unlocking requires both your master password and an authenticator token.'
                : 'Generate a secret to require an authenticator token when unlocking the vault.'}
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
            <Label for="totp-verification">Step 2 — Confirm a code</Label>
            <Input
              id="totp-verification"
              type="text"
              inputmode="numeric"
              maxlength={TOTP_CODE_LENGTH}
              autocomplete="one-time-code"
              placeholder="Enter 6-digit code"
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
                <AlertTitle>Unable to generate secret</AlertTitle>
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
              {isConfirmingTotp ? 'Verifying…' : 'Verify & enable'}
            </Button>
            <Button
              type="button"
              variant="outline"
              class="gap-2"
              onclick={handleGenerateTotpSecret}
              disabled={isGeneratingTotpSecret}
            >
              <RefreshCw class={`h-4 w-4 ${isGeneratingTotpSecret ? 'animate-spin' : ''}`} aria-hidden="true" />
              Generate another secret
            </Button>
            <Button type="button" variant="ghost" onclick={cancelTotpSetup}>Cancel</Button>
          </div>

          <p class="text-xs text-muted-foreground">
            Codes rotate every 30 seconds. If verification fails, wait for the next code before trying again.
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
                  ? 'Copy the secret if you need to enrol another authenticator or keep an offline backup.'
                  : 'This device does not have a local copy of the secret. Rotate the secret to capture it again.'}
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
                  Copy secret
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  class="gap-2"
                  onclick={() => copyProvisioningUri(currentProvisioningUri, 'stored')}
                  disabled={!currentProvisioningUri}
                >
                  <Link2 class="h-4 w-4" aria-hidden="true" />
                  Copy setup link
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
                <AlertTitle>No local secret available</AlertTitle>
                <AlertDescription>Rotate the secret to store a copy on this device for backup access.</AlertDescription>
              </div>
            </Alert>
          {/if}
        </div>
      {/if}
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Lock class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Master Password &amp; Encryption</CardTitle>
        <CardDescription>Manage the master password and key derivation policy.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-5 pt-4">
      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-muted/20 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p class="text-sm font-semibold text-foreground">Master Password</p>
          <p class="text-sm text-muted-foreground">Update the password used to unlock your vault.</p>
        </div>
        <Button variant="outline" onclick={openPasswordModal}>
          Change Password
        </Button>
      </div>

      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-muted/20 px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p class="text-sm font-semibold text-foreground">Key Derivation</p>
          <p class="text-sm text-muted-foreground">
            {argon2Loading ? 'Loading key derivation parameters…' : argon2Summary}
          </p>
        </div>
        <Button variant="outline" size="sm" onclick={openKdfModal}>
          Reconfigure KDF
        </Button>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <CalendarClock class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Auto-lock Controls</CardTitle>
        <CardDescription>Define when the vault should automatically lock itself.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">Lock on Suspend</p>
          <p class="text-sm text-muted-foreground">Lock whenever the system sleeps or hibernates.</p>
        </div>
        <Switch
          checked={currentSettings.lockOnSuspend}
          aria-label="Toggle lock on suspend"
          onclick={() => toggleSetting('lockOnSuspend')}
        />
      </div>

      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">Lock on Minimise</p>
          <p class="text-sm text-muted-foreground">Lock the vault when the window is minimised.</p>
        </div>
        <Switch
          checked={currentSettings.lockOnMinimize}
          aria-label="Toggle lock on minimise"
          onclick={() => toggleSetting('lockOnMinimize')}
        />
      </div>

      <div class="flex flex-col gap-2 rounded-lg border border-border/60 bg-muted/20 px-4 py-4">
        <Label class="text-sm font-semibold text-foreground">Auto-lock After Inactivity</Label>
        <p class="text-sm text-muted-foreground">Lock the vault automatically after the selected idle period.</p>
        <Select type="single" value={currentSettings.autoLockInactivity} onValueChange={updateAutoLock}>
          <SelectTrigger aria-label="Select auto-lock inactivity" class="w-full sm:w-56">
            <span data-slot="select-value" class="truncate text-sm">
              {autoLockOptions.find((option) => option.value === currentSettings.autoLockInactivity)?.label ?? 'Select duration'}
            </span>
          </SelectTrigger>
          <SelectContent>
            {#each autoLockOptions as option}
              <SelectItem value={option.value}>{option.label}</SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Fingerprint class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Biometric &amp; Session</CardTitle>
        <CardDescription>Control biometric unlock availability and session persistence.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">Biometric Unlock</p>
          <p class="text-sm text-muted-foreground">Allow fingerprint or face recognition to unlock the vault.</p>
        </div>
        <Switch
          checked={currentSettings.biometricUnlock}
          aria-label="Toggle biometric unlock"
          onclick={() => toggleSetting('biometricUnlock')}
        />
      </div>

      <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
        <div>
          <p class="text-sm font-semibold text-foreground">Session Persistence</p>
          <p class="text-sm text-muted-foreground">Remember the unlocked session between restarts.</p>
        </div>
        <Switch
          checked={currentSettings.sessionPersistence}
          aria-label="Toggle session persistence"
          onclick={() => toggleSetting('sessionPersistence')}
        />
      </div>
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <MonitorSmartphone class="h-5 w-5" aria-hidden="true" />
      </div>
      <div class="flex w-full flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <CardTitle>Paired Devices</CardTitle>
          <CardDescription>Review devices authorised for biometric or key-based unlock.</CardDescription>
        </div>
        <Button variant="outline" size="sm" onclick={handlePairDevice} disabled={devicesLoading}>
          Pair New Device
        </Button>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      {#if devicesLoading}
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 class="h-4 w-4 animate-spin" aria-hidden="true" />
          <span>Loading devices…</span>
        </div>
      {:else if devices.length === 0}
        <p class="text-sm text-muted-foreground">No devices have been paired yet.</p>
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
                  {device.lastSeen ?? 'No recent activity'}{device.isCurrent ? ' • Current device' : ''}
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
            Revoke All Devices
          </Button>
        </div>
      {/if}
    </CardContent>
  </Card>

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <EyeOff class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Privacy Controls</CardTitle>
        <CardDescription>Fine-tune privacy and diagnostic data handling.</CardDescription>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-5 pt-4">
      <div class="flex flex-col gap-3">
        {#each privacyToggles as toggle}
          <div class="flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-3">
            <div>
              <p class="text-sm font-semibold text-foreground">{toggle.title}</p>
              <p class="text-sm text-muted-foreground">{toggle.description}</p>
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
          Access Local Logs
        </Button>
        <Button variant="outline" class="justify-start gap-3" onclick={() => {}}>
          <Trash2 class="h-4 w-4" aria-hidden="true" />
          Clear Local Logs
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

  <Card class="border-border/60 bg-background/80 backdrop-blur">
    <CardHeader class="flex flex-row items-start gap-3 border-b border-border/40 pb-4">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Shield class="h-5 w-5" aria-hidden="true" />
      </div>
      <div>
        <CardTitle>Security Actions</CardTitle>
        <CardDescription>Execute advanced maintenance and security tasks.</CardDescription>
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
            <div>
              <p class="text-sm font-semibold text-foreground">{action.title}</p>
              <p class="text-sm text-muted-foreground">{action.description}</p>
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
      <DialogTitle>Change Master Password</DialogTitle>
      <DialogDescription>
        Provide your current master password and enter a new secure password to re-encrypt the vault.
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      <div class="space-y-2">
        <Label for="current-password">Current Password</Label>
        <div class="relative">
          <Input
            id="current-password"
            type={showCurrentPassword ? 'text' : 'password'}
            placeholder="Enter current password"
            bind:value={currentPassword}
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute right-1 top-1/2 -translate-y-1/2"
            aria-label={showCurrentPassword ? 'Hide current password' : 'Show current password'}
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
        <Label for="new-password">New Password</Label>
        <Input
          id="new-password"
          type="password"
          placeholder="Enter new password"
          bind:value={newPassword}
        />
      </div>

      <div class="space-y-2">
        <Label for="confirm-password">Confirm New Password</Label>
        <Input
          id="confirm-password"
          type="password"
          placeholder="Confirm new password"
          bind:value={confirmPassword}
        />
      </div>

      <div class="flex items-start gap-3 rounded-lg border border-destructive/40 bg-destructive/10 p-3 text-sm text-destructive">
        <TriangleAlert class="mt-0.5 h-4 w-4" aria-hidden="true" />
        <p>Changing the master password re-encrypts the vault. The operation may take several minutes for large vaults.</p>
      </div>
      {#if changePasswordError}
        <p class="text-sm text-destructive">{changePasswordError}</p>
      {/if}
    </div>

    <DialogFooter class="gap-2">
      <Button type="button" variant="outline" onclick={closePasswordModal}>
        Cancel
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
        Change Password
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>

<Dialog open={kdfModalOpen} onOpenChange={handleKdfDialogChange}>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>Reconfigure Key Derivation</DialogTitle>
      <DialogDescription>
        Adjust the Argon2id parameters used when deriving the vault encryption key.
      </DialogDescription>
    </DialogHeader>

    <div class="space-y-4">
      <div class="grid gap-3 sm:grid-cols-3">
        <div class="space-y-2">
          <Label for="kdf-memory">Memory (MiB)</Label>
          <Input id="kdf-memory" type="number" min="8" bind:value={kdfMemoryMb} />
        </div>
        <div class="space-y-2">
          <Label for="kdf-time">Time Cost</Label>
          <Input id="kdf-time" type="number" min="1" bind:value={kdfTimeCost} />
        </div>
        <div class="space-y-2">
          <Label for="kdf-parallelism">Parallelism</Label>
          <Input id="kdf-parallelism" type="number" min="1" bind:value={kdfParallelism} />
        </div>
      </div>

      <div class="space-y-2">
        <Label for="kdf-password">Current Password</Label>
        <div class="relative">
          <Input
            id="kdf-password"
            type={showKdfPassword ? 'text' : 'password'}
            placeholder="Enter current password"
            bind:value={kdfCurrentPassword}
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="absolute right-1 top-1/2 -translate-y-1/2"
            aria-label={showKdfPassword ? 'Hide current password' : 'Show current password'}
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
        <p>Updating Argon2 parameters will re-encrypt the vault and may take a few moments.</p>
      </div>

      {#if kdfError}
        <p class="text-sm text-destructive">{kdfError}</p>
      {/if}
    </div>

    <DialogFooter class="gap-2">
      <Button type="button" variant="outline" onclick={() => handleKdfDialogChange(false)}>
        Cancel
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
        Apply Changes
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
