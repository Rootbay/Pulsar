<script lang="ts">
    import { onMount } from 'svelte';
    import { writable } from 'svelte/store';
    import { vaultSettings } from '$lib/stores/vault';
    import type { VaultSettings } from '$lib/config/settings';
    import Switch from '$lib/components/ui/Switch.svelte';
    import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
    import { iconPaths } from '$lib/icons';

    interface Vault {
        id: number;
        name: string;
        items: number;
        size: string;
        lastModified: string;
        status: string;
        encrypted: boolean;
        path: string;
        settings: VaultSettings;
    }

    const defaultVaults: Vault[] = [
        { 
            id: 1, name: 'Personal Vault', items: 127, size: '2.3 MB', lastModified: '2 hours ago', status: 'unlocked', encrypted: true, path: '/Users/username/Documents/PersonalVault.db',
            settings: { name: 'Personal Vault', totp: true, backups: false, compression: false }
        },
        { 
            id: 2, name: 'Work Vault', items: 89, size: '1.8 MB', lastModified: '1 day ago', status: 'encrypted', encrypted: true, path: '/Users/username/Documents/WorkVault.db',
            settings: { name: 'Work Vault', totp: true, backups: true, compression: true }
        },
        { 
            id: 3, name: 'Family Vault', items: 34, size: '0.9 MB', lastModified: '3 days ago', status: 'encrypted', encrypted: true, path: '/Users/username/Documents/FamilyVault.db',
            settings: { name: 'Family Vault', totp: false, backups: true, compression: false }
        }
    ];

    let vaults = writable<Vault[]>([...defaultVaults]);
    let selectedVault = writable<Vault>(defaultVaults[0]);

    $: {
        if ($selectedVault) {
            (vaultSettings as any).loadSettings({ ...$selectedVault.settings, name: $selectedVault.name });
        }
    }

    function selectVault(vault: Vault) {
        selectedVault.set(vault);
    }

    function toggleSetting(settingKey: keyof VaultSettings) {
        vaultSettings.update(current => {
            if (current) {
                return {
                    ...current,
                    [settingKey]: !current[settingKey]
                };
            }
            return current;
        });
    }

    function handleVaultNameChange(event: Event) {
        const target = event.target as HTMLInputElement;
        vaultSettings.update(current => {
            if (current) {
                return {
                    ...current,
                    name: target.value
                };
            }
            return current;
        });
    }

    onMount(() => {
        (vaultSettings as any).loadSettings({ ...defaultVaults[0].settings, name: defaultVaults[0].name });
    });

    $: totalItems = $vaults.reduce((sum, vault) => sum + vault.items, 0);
    const weakPasswords = 23; 
    const duplicates = 5; 
</script>

<div class="page-container">
    <div class="content-wrapper">

        <SettingsCard icon={iconPaths.folder} title="Vault Management" description="Manage your password vaults.">
            <div class="management-grid">
                <div>
                    <div class="vault-list-header">
                        <h3 class="vault-list-title">Available Vaults</h3>
                        <button class="btn btn-primary btn-sm">
                            <svg class="icon icon-sm" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></svg>
                            Create New
                        </button>
                    </div>

                    <div class="vault-list">
                        {#each $vaults as vault (vault.id)}
                            <button class="vault-item" class:selected={$selectedVault && $selectedVault.id === vault.id} on:click={() => selectVault(vault)}>
                                <div class="vault-info">
                                    <div class="vault-details">
                                        <h3>{vault.name}</h3>
                                        <div class="vault-meta">{vault.items} items • {vault.size} • Last modified: {vault.lastModified}</div>
                                    </div>
                                    <div class="vault-status-indicators">
                                        <span class="status-badge status-encrypted">Encrypted</span>
                                        {#if vault.status === 'unlocked'}
                                            <span class="status-badge status-unlocked">Unlocked</span>
                                        {/if}
                                    </div>
                                </div>
                            </button>
                        {/each}
                    </div>

                    <div class="list-actions">
                        <button class="btn btn-secondary btn-sm" style="flex: 1;">
                            <svg class="icon icon-sm" viewBox="0 0 24 24"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/></svg>
                            Import
                        </button>
                        <button class="btn btn-secondary btn-sm" style="flex: 1;">
                            <svg class="icon icon-sm" viewBox="0 0 24 24"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>
                            Merge
                        </button>
                    </div>
                </div>

                <div>
                    <h3 class="vault-list-title">Vault Information</h3>

                    {#if selectedVault}
                        <div class="details-form-grid">
                            <div class="form-group">
                                <label class="form-label" for="vault-name">Vault Name</label>
                                <input id="vault-name" type="text" class="form-input" bind:value={$vaultSettings.name} on:input={handleVaultNameChange}>
                            </div>
                            <div class="form-group">
                                <label class="form-label" for="vault-status">Status</label>
                                <input id="vault-status" type="text" class="form-input" value="{$selectedVault.encrypted ? 'Encrypted' : 'Unencrypted'} & {$selectedVault.status === 'unlocked' ? 'Unlocked' : 'Locked'}" readonly>
                            </div>
                        </div>

                        <div class="form-group">
                            <label class="form-label" for="vault-path">Vault Path</label>
                            <div class="input-group">
                                <input id="vault-path" type="text" class="form-input" value={$selectedVault.path} readonly style="flex: 1;">
                                <button class="btn btn-secondary btn-sm">
                                    <svg class="icon icon-sm" viewBox="0 0 24 24"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                                    Open
                                </button>
                            </div>
                        </div>

                        <div class="encryption-details-box">
                            <h4 style="font-weight: 600; margin-bottom: 0.75rem;">Encryption Details</h4>
                            <div class="encryption-details-grid">
                                <div><p style="color: hsl(var(--muted-foreground));">Algorithm</p><p>AES-256-GCM</p></div>
                                <div><p style="color: hsl(var(--muted-foreground));">KDF</p><p>Argon2id</p></div>
                                <div><p style="color: hsl(var(--muted-foreground));">Memory (MB)</p><p>64</p></div>
                                <div><p style="color: hsl(var(--muted-foreground));">Iterations</p><p>3</p></div>
                            </div>
                        </div>
                    {/if}
                </div>
            </div>
        </SettingsCard>

        <SettingsCard icon={iconPaths.settings} title="Vault-Level Settings" description="Configure settings specific to the selected vault.">
            {#if $selectedVault}
            <div class="settings-list">
                <div class="toggle-group">
                    <div class="toggle-content">
                        <div class="toggle-label">Per-Entry TOTP Storage</div>
                        <div class="toggle-description">Enable storage of 2FA codes within individual password entries</div>
                    </div>
                    <Switch checked={$vaultSettings.totp} ariaLabel="Toggle per-entry TOTP storage" on:click={() => toggleSetting('totp')} />
                </div>

                <div class="toggle-group">
                    <div class="toggle-content">
                        <div class="toggle-label">Automatic Backups</div>
                        <div class="toggle-description">Enable automatic encrypted backups of this vault</div>
                    </div>
                    <Switch checked={$vaultSettings.backups} ariaLabel="Toggle automatic backups" on:click={() => toggleSetting('backups')} />
                </div>

                <div class="toggle-group">
                    <div class="toggle-content">
                        <div class="toggle-label">Compression</div>
                        <div class="toggle-description">Compress vault data to save disk space</div>
                    </div>
                    <Switch checked={$vaultSettings.compression} ariaLabel="Toggle compression" on:click={() => toggleSetting('compression')} />
                </div>
            </div>
            {/if}
        </SettingsCard>

        <SettingsCard icon={iconPaths.key} title="Vault Actions" description="Perform various actions on your vault.">
            <div class="vault-actions-grid">
                <button class="btn btn-secondary btn-action">
                    <svg class="icon" viewBox="0 0 24 24"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><path d="M22 4L12 14.01l-3-3"/></svg>
                    <span>Integrity Check</span>
                    <span class="btn-action-label">Verify vault consistency</span>
                </button>

                <button class="btn btn-secondary btn-action">
                    <svg class="icon" viewBox="0 0 24 24"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3"/></svg>
                    <span>Compact Database</span>
                    <span class="btn-action-label">Optimize file size</span>
                </button>

                <button class="btn btn-secondary btn-action">
                    <svg class="icon" viewBox="0 0 24 24"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14,2 14,8 20,8"/></svg>
                    <span>Export Header</span>
                    <span class="btn-action-label">Backup metadata</span>
                </button>

                <button class="btn btn-secondary btn-action">
                    <svg class="icon" viewBox="0 0 24 24"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><circle cx="12" cy="16" r="1"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                    <span>Re-encrypt Vault</span>
                    <span class="btn-action-label">Change encryption keys</span>
                </button>

                <button class="btn btn-secondary btn-action">
                    <svg class="icon" viewBox="0 0 24 24"><path d="M3 3v5h5M3 8a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 4"/><path d="M21 21v-5h-5M21 16a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 20"/></svg>
                    <span>Security Analysis</span>
                    <span class="btn-action-label">Check password strength</span>
                </button>

                <button class="btn btn-destructive btn-action">
                    <svg class="icon" viewBox="0 0 24 24"><path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                    <span>Delete Vault</span>
                    <span class="btn-action-label-destructive">Permanent removal</span>
                </button>
            </div>
        </SettingsCard>

        <SettingsCard icon={iconPaths.paper} title="Vault Statistics" description="View statistics about your vault.">
            <div class="stats-grid">
                <div class="stat-item"><div class="stat-value">{totalItems}</div><div class="stat-label">Total Items</div></div>
                <div class="stat-item">
                    <div class="stat-value" class:placeholder-value={!$selectedVault} style={$selectedVault ? 'color: hsl(var(--success));' : ''}>
                        {$selectedVault ? $selectedVault.size : 'N/A'}
                    </div>
                    <div class="stat-label">File Size</div>
                </div>
                <div class="stat-item"><div class="stat-value" style="color: hsl(var(--warning));">{weakPasswords}</div><div class="stat-label">Weak Passwords</div></div>
                <div class="stat-item"><div class="stat-value" style="color: hsl(var(--destructive));">{duplicates}</div><div class="stat-label">Duplicates</div></div>
            </div>
        </SettingsCard>
    </div>
</div>

<style>
    

    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }

    .page-container {
        flex: 1;
        padding: 32px;
        overflow-y: auto;
    }
    .content-wrapper {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }
    .management-grid {
        display: grid;
        grid-template-columns: 1fr 2fr;
        gap: 1.5rem;
    }
    .settings-list, .vault-actions-grid, .vault-list {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
    }
     .vault-actions-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 1rem;
    }
    .input-group {
        display: flex;
        gap: 0.5rem;
    }
    .details-form-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
        margin-bottom: 10px;
    }
    .encryption-details-box {
        background: hsl(var(--muted) / 0.3);
        padding: 1rem;
        border-radius: var(--radius);
        margin-top: 1rem;
    }
    .encryption-details-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
        font-size: 0.875rem;
    }
     .list-actions {
        margin-top: 1rem;
        display: flex;
        gap: 0.5rem;
    }

    

    .btn {
        padding: 0.375rem 0.75rem;
        border-radius: var(--radius);
        border: none;
        font-size: 0.8rem;
        cursor: pointer;
        transition: var(--transition-smooth);
        font-weight: 500;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.375rem;
    }
    .btn-primary {
        background: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
    }
    .btn-primary:hover {
        background: hsl(var(--primary) / 0.9);
    }
    .btn-secondary {
        background: hsl(var(--secondary));
        color: hsl(var(--secondary-foreground));
        border: 1px solid hsl(var(--border));
    }
    .btn-secondary:hover {
        background: hsl(var(--muted));
    }
    .btn-sm {
        padding: 0.25rem 0.5rem;
        font-size: 0.75rem;
    }
    .btn-destructive {
        background: hsl(var(--destructive));
        color: hsl(var(--destructive-foreground));
    }
    .btn-destructive:hover {
        background: hsl(var(--destructive) / 0.9);
    }
    .btn-action { 
        height: auto;
        padding: 1rem;
        flex-direction: column;
        gap: 0.5rem;
    }
    .btn-action-label {
        font-size: 0.75rem;
        color: hsl(var(--muted-foreground));
    }
    .btn-action-label-destructive {
        font-size: 0.75rem;
        color: hsl(var(--destructive-foreground));
    }

    .vault-list-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.75rem;
    }
    .vault-list-title {
        font-weight: 600;
    }
    .vault-item {

        background: hsl(var(--background));
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        padding: 0.75rem;
        cursor: pointer;
        transition: var(--transition-smooth);
        width: 100%;
        text-align: left;
        color: inherit;
        font-family: inherit; 
    }
    .vault-item:hover {
        background: hsl(var(--muted) / 0.5);
    }
    .vault-item.selected {
        border-color: hsl(var(--primary));
        background: hsl(var(--primary) / 0.1);
    }
    .vault-info {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    .vault-details h3 {
        font-size: 0.875rem;
        font-weight: 600;
        color: hsl(var(--foreground));
        margin-bottom: 0.125rem;
    }
    .vault-meta {
        font-size: 0.75rem;
        color: hsl(var(--muted-foreground));
    }
    .vault-status-indicators {
        display: flex;
        flex-direction: column;
        align-items: end;
        gap: 0.25rem;
    }
    .status-badge {
        padding: 0.0625rem 0.375rem;
        border-radius: 0.5rem;
        font-size: 0.625rem;
        font-weight: 500;
    }
    .status-encrypted {
        background: hsl(var(--success) / 0.1);
        color: hsl(var(--success));
    }
    .status-unlocked {
        background: hsl(var(--primary) / 0.1);
        color: hsl(var(--primary));
    }

    .form-group {
        margin-bottom: 1rem;
    }
    .form-label {
        display: block;
        font-size: 0.875rem;
        color: hsl(var(--foreground));
        margin-bottom: 0.375rem;
        font-weight: 500;
    }
    .form-input {
        width: 100%;
        padding: 0.5rem 0.625rem;
        background: hsl(var(--input));
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        color: hsl(var(--foreground));
        font-size: 0.8rem;
        transition: var(--transition-smooth);
    }
    .form-input:focus {
        outline: none;
        border-color: hsl(var(--ring));
        box-shadow: 0 0 0 2px hsl(var(--ring) / 0.2);
    }
    .form-input:read-only {
        background: hsl(var(--muted));
        color: hsl(var(--muted-foreground));
    }

    .toggle-group {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    .toggle-content .toggle-label {
        font-size: 0.875rem;
        color: hsl(var(--foreground));
        font-weight: 500;
    }
    .toggle-content .toggle-description {
        font-size: 0.8rem;
        color: hsl(var(--muted-foreground));
        margin-top: 0.125rem;
    }
    

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 1rem;
    }
    .stat-item {
        background: hsl(var(--muted) / 0.3);
        padding: 0.75rem;
        border-radius: var(--radius);
        border: 1px solid hsl(var(--border));
        text-align: center;
    }
    .stat-value {
        font-size: 1.25rem;
        font-weight: 700;
        color: hsl(var(--primary));
    }
    .stat-label {
        font-size: 0.7rem;
        color: hsl(var(--muted-foreground));
        margin-top: 0.125rem;
    }

    .icon {
        width: 1rem;
        height: 1rem;
        stroke: currentColor;
        fill: none;
        stroke-width: 2;
    }
    .icon-sm {
        width: 0.875rem;
        height: 0.875rem;
    }

    @media (max-width: 1024px) {
        .stats-grid {
            grid-template-columns: repeat(2, 1fr);
        }
        .management-grid {
            grid-template-columns: 1fr;
        }
    }
    @media (max-width: 640px) {
        .stats-grid {
            grid-template-columns: 1fr;
        }
        .vault-actions-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
