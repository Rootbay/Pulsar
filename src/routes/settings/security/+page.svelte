<script lang="ts">
  import { get } from 'svelte/store';
  import { onDestroy } from 'svelte';
  import { securitySettings } from '$lib/stores/security';
  import type { SecuritySettings } from '$lib/config/settings';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Switch } from '$lib/components/ui/switch';
  import { Badge } from '$lib/components/ui/badge';
  import { cn } from '$lib/utils';
  import { Lock, RefreshCw, Trash2, ShieldCheck, Fingerprint, Smartphone, CalendarClock, MonitorSmartphone, EyeOff, Eye, TriangleAlert, HardDrive, Shield } from '@lucide/svelte';
  
  type BooleanSettingKey = {
    [K in keyof SecuritySettings]: SecuritySettings[K] extends boolean ? K : never;
  }[keyof SecuritySettings];

  type AutoLockOption = {
    value: SecuritySettings['autoLockInactivity'];
    label: string;
  };

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

  const pairedDevices: Array<{
    name: string;
    lastSeen: string;
    isCurrent: boolean;
    type: string;
    Icon: typeof Fingerprint;
  }> = [
    {
      name: 'Touch ID (MacBook Pro)',
      lastSeen: '2 minutes ago',
      isCurrent: true,
      type: 'Biometric',
      Icon: Fingerprint
    },
    {
      name: 'Pixel 8 (Device Key)',
      lastSeen: 'Yesterday � 20:14',
      isCurrent: false,
      type: 'Device Key',
      Icon: Smartphone
    }
  ];

  const securityActions: Array<{
    title: string;
    description: string;
    Icon: typeof RefreshCw;
  }> = [
    {
      title: 'Re-key Vault',
      description: 'Rotate encryption keys and re-encrypt stored data.',
      Icon: RefreshCw
    },
    {
      title: 'Clear Memory',
      description: 'Scrub sensitive material from memory immediately.',
      Icon: Trash2
    },
    {
      title: 'Integrity Check',
      description: 'Verify vault contents for tampering or corruption.',
      Icon: ShieldCheck
    }
  ];

  let currentSettings: SecuritySettings = get(securitySettings);
  let passwordModalOpen = false;
  let showCurrentPassword = false;
  let currentPassword = '';
  let newPassword = '';
  let confirmPassword = '';

  const unsubscribe = securitySettings.subscribe((settings) => {
    currentSettings = settings;
  });

  onDestroy(() => {
    unsubscribe();
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

  function openPasswordModal() {
    passwordModalOpen = true;
    currentPassword = '';
    newPassword = '';
    confirmPassword = '';
    showCurrentPassword = false;
  }

  function closePasswordModal() {
    passwordModalOpen = false;
  }

  function handleDialogChange(open: boolean) {
    passwordModalOpen = open;
    if (!passwordModalOpen) {
      showCurrentPassword = false;
    }
  }

  function togglePasswordVisibility() {
    showCurrentPassword = !showCurrentPassword;
  }
</script>

<div class="flex flex-1 flex-col gap-6 overflow-y-auto px-8 py-8">
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
          <p class="text-sm text-muted-foreground">Argon2id � memory 64&nbsp;MB � time cost 3 � parallelism 4</p>
        </div>
        <Button variant="outline" size="sm" onclick={() => {}}>
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
        <Button variant="outline" size="sm" onclick={() => {}}>
          Pair New Device
        </Button>
      </div>
    </CardHeader>
    <CardContent class="flex flex-col gap-4 pt-4">
      {#each pairedDevices as device}
        <div
          class={cn(
            'flex items-start justify-between gap-4 rounded-lg border border-border/60 bg-muted/20 px-4 py-4 sm:items-center',
            device.isCurrent ? 'border-primary/60 bg-primary/10' : ''
          )}
        >
          <div class="flex items-start gap-3">
            <div class="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
              <device.Icon class="h-5 w-5" aria-hidden="true" />
            </div>
            <div>
              <p class="text-sm font-semibold text-foreground">{device.name}</p>
              <p class="text-xs text-muted-foreground">
                {device.lastSeen}{device.isCurrent ? ' � Current device' : ''}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <Badge variant="secondary">{device.type}</Badge>
            <Button variant="ghost" size="icon" onclick={() => {}} aria-label={`Remove ${device.name}`}>
              <Trash2 class="h-4 w-4" aria-hidden="true" />
            </Button>
          </div>
        </div>
      {/each}

      <div class="flex justify-end">
        <Button variant="destructive" size="sm" onclick={() => {}}>
          Revoke All Devices
        </Button>
      </div>
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
            onclick={() => {}}
          >
            <action.Icon class="h-5 w-5 text-primary" aria-hidden="true" />
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
    </div>

    <DialogFooter class="gap-2">
      <Button type="button" variant="outline" onclick={closePasswordModal}>
        Cancel
      </Button>
      <Button type="button" variant="destructive" onclick={() => {}}>
        Change Password
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
