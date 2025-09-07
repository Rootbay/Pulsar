<script lang="ts">
	
	import { securitySettings } from '$lib/stores/security';
	import type { SecuritySettings } from '$lib/config/settings';
	import Switch from '$lib/components/ui/Switch.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
	import { iconPaths } from '$lib/icons';

	let currentSecuritySettings: SecuritySettings;
	securitySettings.subscribe(value => {
		currentSecuritySettings = value;
	});

	let passwordModalOpen = false;
	let showCurrentPassword = false;

	function updateSettings(newValues: Partial<SecuritySettings>) {
		securitySettings.set({ ...currentSecuritySettings, ...newValues });
	}

	function toggleSwitch(setting: keyof SecuritySettings) {
		if (typeof currentSecuritySettings[setting] === 'boolean') {
			updateSettings({ [setting]: !currentSecuritySettings[setting] } as Partial<SecuritySettings>);
		}
	}

	function openPasswordModal() {
		passwordModalOpen = true;
	}

	function closePasswordModal() {
		passwordModalOpen = false;
	}

	function togglePasswordVisibility() {
		showCurrentPassword = !showCurrentPassword;
	}

	function handleSelectChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		updateSettings({ autoLockInactivity: target.value });
	}
</script>

<div class="main-content">
  <SettingsCard icon={iconPaths.lock} title="Master Password & Encryption" description="Manage your master password and encryption settings.">
    <div style="margin-bottom: 1.5rem;">
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Master Password</div>
          <div class="setting-description">Change your master password to secure your vault</div>
        </div>
        <button class="btn btn-outline" on:click={openPasswordModal}>Change Password</button>
      </div>
      <div class="kdf-info">
        <div>
          <p style="font-weight: 500;">Key Derivation Function</p>
          <p style="font-size: 0.875rem; color: hsl(var(--muted-foreground));">Argon2id (m=64MB, t=3, p=4)</p>
        </div>
        <button class="btn btn-outline btn-sm">Reconfigure KDF</button>
      </div>
    </div>
  </SettingsCard>

  <SettingsCard icon={iconPaths.calendar} title="Auto-lock Controls" description="Configure automatic locking behavior.">
    <div>
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Lock on Suspend</div>
          <div class="setting-description">Automatically lock when system goes to sleep</div>
        </div>
        <Switch checked={currentSecuritySettings.lockOnSuspend} ariaLabel="Toggle lock on suspend" on:click={() => toggleSwitch('lockOnSuspend')} />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Lock on Minimize</div>
          <div class="setting-description">Lock vault when app is minimized</div>
        </div>
        <Switch checked={currentSecuritySettings.lockOnMinimize} ariaLabel="Toggle lock on minimize" on:click={() => toggleSwitch('lockOnMinimize')} />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Auto-lock After Inactivity</div>
          <div class="setting-description">Lock vault after specified idle time</div>
        </div>
        <Select bind:value={currentSecuritySettings.autoLockInactivity} options={[{value: 'Immediate', label: 'Immediate'}, {value: '1 minute', label: '1 minute'}, {value: '5 minutes', label: '5 minutes'}, {value: '15 minutes', label: '15 minutes'}, {value: 'Custom...', label: 'Custom...'}]} on:change={handleSelectChange} ariaLabel="Select auto-lock inactivity" />
      </div>
    </div>
  </SettingsCard>

  <SettingsCard icon={iconPaths.eye} title="Biometric & Session" description="Manage biometric unlock and session settings.">
    <div>
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Biometric Unlock</div>
          <div class="setting-description">Use fingerprint or face recognition to unlock</div>
        </div>
        <Switch checked={currentSecuritySettings.biometricUnlock} ariaLabel="Toggle biometric unlock" on:click={() => toggleSwitch('biometricUnlock')} />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Session Persistence</div>
          <div class="setting-description">Remember unlock state between app restarts</div>
        </div>
        <Switch checked={currentSecuritySettings.sessionPersistence} ariaLabel="Toggle session persistence" on:click={() => toggleSwitch('sessionPersistence')} />
      </div>
    </div>
  </SettingsCard>

  <SettingsCard icon={iconPaths.default} title="Paired Devices" description="Manage your paired devices.">
    <div slot="header-content" style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem;">
      <h2 class="card-header" style="margin-bottom: 0;">
        <svg class="icon" style="color: hsl(var(--primary))" viewBox="0 0 24 24">
          <rect width="14" height="20" x="5" y="2" rx="2" ry="2"/>
          <path d="M12 18h.01"/>
        </svg>
        Paired Devices
      </h2>
      <button class="btn btn-outline btn-sm">Pair New Device</button>
    </div>
    <div>
      <div class="device-item">
        <div class="device-info">
          <div class="device-icon">
            <svg class="icon" viewBox="0 0 24 24">
              <path d="M12 10a2 2 0 0 0-2 2c0 1.02-.1 2.51-.26 4"/>
              <path d="M14 13.12c0 2.38 0 6.38-1 8.88"/>
              <path d="M17.29 21.02c.12-.6.43-2.3.5-3.02"/>
              <path d="M2 12a10 10 0 0 1 18-6"/>
              <path d="M2 16h.01"/>
              <path d="M21.8 16c.2-2 .131-5.354 0-6"/>
              <path d="M5 19.5C5.5 18 6 15 6 12a6 6 0 0 1 .34-2"/>
              <path d="M8.65 22c.21-.66.45-1.32.57-2"/>
              <path d="M9 6.8a6 6 0 0 1 9 5.2v2"/>
            </svg>
          </div>
          <div class="device-details">
            <div class="device-name">Touch ID (MacBook Pro)</div>
            <div class="device-meta">
              <span>2 minutes ago</span>
              <span>Current device</span>
            </div>
          </div>
        </div>
        <div class="device-actions">
          <span class="badge badge-primary">Biometric</span>
          <button class="btn btn-outline btn-sm" aria-label="Remove device">
            <svg class="icon icon-sm" style="color: hsl(var(--destructive))" viewBox="0 0 24 24">
              <path d="M3 6h18"/>
              <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
              <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
            </svg>
          </button>
        </div>
      </div>
      </div>
    <div style="margin-top: 1rem;">
      <button class="btn btn-destructive btn-sm">Revoke All Devices</button>
    </div>
  </SettingsCard>

  <SettingsCard icon={iconPaths.eyeOff} title="Privacy Controls" description="Manage your privacy settings.">
    <div style="margin-bottom: 1.5rem;">
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">External Breach Check</div>
          <div class="setting-description">Check passwords against known breach databases (opt-in)</div>
        </div>
        <Switch checked={currentSecuritySettings.externalBreachCheck} ariaLabel="Toggle external breach check" on:click={() => toggleSwitch('externalBreachCheck')} />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Local Reuse Detection</div>
          <div class="setting-description">Detect password reuse within your vault</div>
        </div>
        <Switch checked={currentSecuritySettings.localReuseDetection} ariaLabel="Toggle local reuse detection" on:click={() => toggleSwitch('localReuseDetection')} />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">Secure RAM Handling</div>
          <div class="setting-description">Use secure memory allocation for sensitive data</div>
        </div>
        <Switch checked={currentSecuritySettings.secureRAMHandling} ariaLabel="Toggle secure RAM handling" on:click={() => toggleSwitch('secureRAMHandling')} />
      </div>
      <div class="grid grid-2" style="margin-bottom: 1.5rem;">
        <button class="btn btn-outline">
          <svg class="icon" viewBox="0 0 24 24">
            <ellipse cx="12" cy="5" rx="9" ry="3"/>
            <path d="m3 5 9 9 9-9"/>
            <path d="M3 12l9 9 9-9"/>
            <path d="M3 5v14l9-9 9 9V5"/>
          </svg>
          Access Local Logs
        </button>
        <button class="btn btn-outline">
          <svg class="icon" viewBox="0 0 24 24">
            <path d="M3 6h18"/>
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
          </svg>
          Clear Local Logs
        </button>
      </div>
      <div class="data-retention">
        <h4>Data Retention Summary</h4>
        <ul>
          <li>• Vault data: Encrypted locally, never sent to servers</li>
          <li>• Activity logs: Kept locally for 30 days, then purged</li>
          <li>• Crash reports: Anonymous, retained for 90 days</li>
        </ul>
      </div>
    </div>
  </SettingsCard>

  <SettingsCard icon={iconPaths.settings} title="Security Actions" description="Perform security-related actions.">
    <div class="grid grid-3">
      <button class="btn btn-outline btn-card">
        <svg class="icon icon-lg" viewBox="0 0 24 24">
          <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
          <path d="M21 3v5h-5"/>
          <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
          <path d="M8 16H3v5"/>
        </svg>
        <span>Re-key Vault</span>
        <span style="font-size: 0.75rem; color: hsl(var(--muted-foreground));">Generate new encryption keys</span>
      </button>
      <button class="btn btn-outline btn-card">
        <svg class="icon icon-lg" viewBox="0 0 24 24">
          <path d="M3 6h18"/>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
        </svg>
        <span>Clear Memory</span>
        <span style="font-size: 0.75rem; color: hsl(var(--muted-foreground));">Force clear sensitive data</span>
      </button>
      <button class="btn btn-outline btn-card">
        <svg class="icon icon-lg" viewBox="0 0 24 24">
          <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/>
          <path d="m9 12 2 2 4-4"/>
        </svg>
        <span>Integrity Check</span>
        <span style="font-size: 0.75rem; color: hsl(var(--muted-foreground));">Verify vault integrity</span>
      </button>
    </div>
  </SettingsCard>
</div>

{#if passwordModalOpen}
<div class="modal active">
  <div class="modal-content">
    <div class="modal-header">
      <h3 class="modal-title">Change Master Password</h3>
      <p class="modal-description">Enter your current password and choose a new strong master password.</p>
    </div>
    <div class="modal-body">
      <div class="input-group">
        <label class="label" for="currentPassword">Current Password</label>
        <div class="input-with-button">
          <input type={showCurrentPassword ? 'text' : 'password'} class="input" placeholder="Enter current password" id="currentPassword">
          <button type="button" class="input-button" aria-label="Toggle password visibility" on:click={togglePasswordVisibility}>
            <svg class="icon icon-sm" viewBox="0 0 24 24">
              <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z"/>
              <circle cx="12" cy="12" r="3"/>
            </svg>
          </button>
        </div>
      </div>
      <div class="input-group">
        <label class="label" for="newPassword">New Password</label>
        <input type="password" class="input" placeholder="Enter new password" id="newPassword">
      </div>
      <div class="input-group">
        <label class="label" for="confirmPassword">Confirm New Password</label>
        <input type="password" class="input" placeholder="Confirm new password" id="confirmPassword">
      </div>
      <div class="warning-box">
        <strong>Warning:</strong> Changing your master password will re-encrypt your entire vault.
        This process may take several minutes.
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-outline" on:click={closePasswordModal}>Cancel</button>
      <button class="btn btn-warning">Change Password</button>
    </div>
  </div>
</div>
{/if}

<style>
  

  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  .main-content {
    flex: 1;
    padding: 32px;
    overflow-y: auto;
    position: relative;
  }

  

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }
  .setting-row:last-child {
    margin-bottom: 0;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .setting-label {
    font-weight: 500;
  }
  .setting-description {
    font-size: 0.875rem;
    color: hsl(var(--muted-foreground));
  }

  

  

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: calc(var(--radius));
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    text-decoration: none;
    border: none;
  }

  .btn-outline {
    background: transparent;
    border: 1px solid hsl(var(--border));
    color: hsl(var(--foreground));
  }

  .btn-outline:hover {
    background: hsl(var(--accent));
  }

  .btn-destructive {
    background: hsl(var(--destructive));
    color: hsl(var(--destructive-foreground));
    border: 1px solid hsl(var(--destructive));
  }

  .btn-destructive:hover {
    background: hsl(var(--destructive) / 0.9);
  }

  .btn-warning {
    background: hsl(var(--warning));
    color: hsl(var(--warning-foreground));
    border: 1px solid hsl(var(--warning));
  }

  .btn-sm {
    padding: 0.25rem 0.75rem;
    font-size: 0.75rem;
  }

  .btn-card {
    height: auto;
    padding: 1rem;
    flex-direction: column;
    text-align: center;
  }

  .kdf-info {
    background: hsl(var(--muted) / 0.3);
    border-radius: calc(var(--radius));
    padding: 1rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .device-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border: 1px solid hsl(var(--border));
    border-radius: calc(var(--radius));
    margin-bottom: 0.75rem;
  }

  .device-item:last-child {
    margin-bottom: 0;
  }

  .device-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .device-icon {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 0.5rem;
    background: hsl(var(--primary) / 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    color: hsl(var(--primary));
  }

  .device-details {
    display: flex;
    flex-direction: column;
  }

  .device-name {
    font-weight: 500;
    margin-bottom: 0.25rem;
  }

  .device-meta {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.875rem;
    color: hsl(var(--muted-foreground));
  }

  .device-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .badge {
    padding: 0.125rem 0.5rem;
    border-radius: calc(var(--radius) - 2px);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .badge-primary {
    background: hsl(var(--primary));
    color: hsl(var(--primary-foreground));
  }

  .grid {
    display: grid;
    gap: 1rem;
  }

  .grid-2 {
    grid-template-columns: repeat(2, 1fr);
  }

  .grid-3 {
    grid-template-columns: repeat(3, 1fr);
  }

  .warning-box {
    padding: 0.75rem;
    background: hsl(var(--warning) / 0.1);
    border: 1px solid hsl(var(--warning) / 0.2);
    border-radius: calc(var(--radius));
    font-size: 0.875rem;
    color: hsl(var(--warning-foreground));
  }

  .data-retention {
    background: hsl(var(--muted) / 0.3);
    border-radius: calc(var(--radius));
    padding: 0.75rem;
  }

  .data-retention h4 {
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  .data-retention ul {
    list-style: none;
    font-size: 0.875rem;
    color: hsl(var(--muted-foreground));
  }

  .data-retention li {
    margin-bottom: 0.25rem;
  }

  .icon {
    width: 1.25rem;
    height: 1.25rem;
    stroke: currentColor;
    fill: none;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .icon-sm {
    width: 1rem;
    height: 1rem;
  }

  .icon-lg {
    width: 1.5rem;
    height: 1.5rem;
  }

  .modal {
    display: none;
    position: fixed;
    inset: 0;
    background: hsl(var(--background) / 0.8);
    backdrop-filter: blur(8px);
    z-index: 100;
    align-items: center;
    justify-content: center;
  }

  .modal.active {
    display: flex;
  }

  .modal-content {
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: calc(var(--radius) + 4px);
    padding: 1.5rem;
    width: 90%;
    max-width: 28rem;
    max-height: 90vh;
    overflow-y: auto;
  }

  .modal-header {
    margin-bottom: 1rem;
  }

  .modal-title {
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .modal-description {
    font-size: 0.875rem;
    color: hsl(var(--muted-foreground));
  }
  .modal-body {
    margin-bottom: 1.5rem;
  }

  .input-group {
    margin-bottom: 1rem;
  }

  .input-group:last-child {
    margin-bottom: 0;
  }

  .label {
    display: block;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  .input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: hsl(var(--background) / 0.5);
    border: 1px solid hsl(var(--border));
    border-radius: calc(var(--radius));
    color: hsl(var(--foreground));
    font-size: 0.875rem;
  }

  .input:focus {
    outline: none;
    border-color: hsl(var(--ring));
    box-shadow: 0 0 0 2px hsl(var(--ring) / 0.2);
  }

  .input-with-button {
    position: relative;
  }

  .input-button {
    position: absolute;
    right: 0.5rem;
    top: 50%;
    transform: translateY(-50%);
    background: transparent;
    border: none;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    padding: 0.25rem;
  }

  .modal-footer {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  

  @media (max-width: 768px) {
    .main-content {
      padding: 1rem;
    }

    .grid-2,
    .grid-3 {
      grid-template-columns: 1fr;
    }

    .setting-row {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.75rem;
    }

    .device-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.75rem;
    }
  }
</style>