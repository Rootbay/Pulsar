<script lang="ts">
    import { slide, fade } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import { onMount } from 'svelte';
    import { backupSettings } from '$lib/stores/backup';
    import Switch from '$lib/components/ui/Switch.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
    import { iconPaths } from '$lib/icons';
    import { invoke } from '@tauri-apps/api/core';

    let automaticBackups: boolean;
    let backupFrequency: string;
    let retentionCount: number;
    let enablePlaintextExport: boolean;
    let syncMode: string;
    let selectedProvider: string | null;
    let exportFormat: string = 'json';

    backupSettings.subscribe(value => {
        automaticBackups = value.automaticBackups;
        backupFrequency = value.backupFrequency;
        retentionCount = value.retentionCount;
        enablePlaintextExport = value.enablePlaintextExport;
        syncMode = value.syncMode;
        selectedProvider = value.selectedProvider;
    });

    let showModal = false;
    let modalContent: {
        title: string;
        description: string;
        type: 'confirm' | 'provider';
        provider: string | null;
        onConfirm: () => void;
    } = {
        title: '',
        description: '',
        type: 'confirm',
        provider: null,
        onConfirm: () => {}
    };

    function showConfirmation(title: string, description: string, onConfirmAction: () => void) {
        modalContent = {
            title,
            description,
            type: 'confirm',
            provider: null,
            onConfirm: () => {
                onConfirmAction();
                showModal = false; 
            }
        };
        showModal = true;
    }

    function openProviderModal(provider: string) {
        backupSettings.update(current => ({ ...current, selectedProvider: provider }));
        modalContent = {
            title: `Setup ${provider.charAt(0).toUpperCase() + provider.slice(1)}`,
            description: `Configure your ${provider} connection settings.`,
            type: 'provider',
            provider: provider,
            onConfirm: () => {
                console.log(`Settings saved for ${provider}`);
                showModal = false;
            }
        };
        showModal = true;
    }

    // In a real application, this would come from a store or API (e.g., vaultStore.getVaultData()).
    // For demonstration, we'll use a dummy string.
    let dummyVaultData = JSON.stringify({
        "passwords": [
            {
                "id": "1",
                "name": "Example Password",
                "username": "user",
                "password": "password123",
                "url": "https://example.com"
            }
        ]
    }, null, 2);

    async function createManualBackup() {
        showConfirmation(
            'Create Manual Backup?',
            'This will create a new encrypted backup of your vault with the current settings.',
            async () => {
                try {
                    const result: string = await invoke('export_vault', { vaultData: dummyVaultData, isPlaintext: false, passphrase: 'users-chosen-password' });
                    alert(result);
                } catch (error) {
                    alert(`Error creating backup: ${error}`);
                }
            }
        );
    }

    async function exportEncrypted() {
        showConfirmation(
            'Export Encrypted Data?',
            'Your vault will be exported in a secure, encrypted format. Keep this file safe.',
            async () => {
                try {
                    const result: string = await invoke('export_vault', { vaultData: dummyVaultData, isPlaintext: false, passphrase: 'users-chosen-password' });
                    alert(result);
                } catch (error) {
                    alert(`Error exporting encrypted data: ${error}`);
                }
            }
        );
    }

    async function exportPlaintext() {
        showConfirmation(
            '⚠️ Export Plaintext Data?',
            'WARNING: This is a high-risk action that will export your data without encryption. Are you sure you want to proceed?',
            async () => {
                try {
                    const result: string = await invoke('export_vault', { vaultData: dummyVaultData, isPlaintext: true, passphrase: 'users-chosen-password' });
                    alert(result);
                } catch (error) {
                    alert(`Error exporting plaintext data: ${error}`);
                }
            }
        );
    }

    async function importData() {
        showConfirmation(
            'Start Import Process?',
            'The import wizard will guide you through selecting a file and mapping your data.',
            async () => {
                try {
                    const importedContent: string = await invoke('import_vault', { passphrase: 'users-chosen-password' });
                    alert(`Data imported successfully:\n\n${importedContent.substring(0, 200)}...`);
                } catch (error) {
                    alert(`Error importing data: ${error}`);
                }
            }
        );
    }

    function handleSwitchChange(setting: 'automaticBackups' | 'enablePlaintextExport') {
        backupSettings.update(current => ({
            ...current,
            [setting]: !current[setting]
        }));
    }

    function handleChange() {
        backupSettings.update(current => ({
            ...current,
            backupFrequency,
            retentionCount,
        }));
    }

    function handleSyncModeChange(mode: string) {
        backupSettings.update(current => ({ ...current, syncMode: mode }));
    }

    onMount(() => {
        // No need to register module here, it's done in backup.ts
        // No need to initialize savedBackupSettings here, it's done in backup.ts
    });
</script>


<div class="settings-container">
    <SettingsCard icon={iconPaths.settings} title="Backups" description="Manage automatic and manual vault backups.">
        <div class="form-group">
            <div class="toggle-container">
                <div class="toggle-info">
                    <label class="form-label" for="automatic-backups-switch">Automatic Backups</label>
                    <div class="form-description">Create backups at regular intervals</div>
                </div>
                <Switch id="automatic-backups-switch" checked={automaticBackups} ariaLabel="Toggle automatic backups" on:click={() => handleSwitchChange('automaticBackups')} />
            </div>
        </div>
        <div class="form-group">
            <label class="form-label" for="backup-frequency">Backup Frequency</label>
            <Select bind:value={backupFrequency} options={[{value: 'daily', label: 'Daily (Default)'}, {value: 'weekly', label: 'Weekly'}, {value: 'custom', label: 'Custom'}]} on:change={handleChange} ariaLabel="Select backup frequency" />
        </div>
        <div class="form-group">
            <label class="form-label" for="retention-count">Retention Count</label>
            <input type="number" class="input" min="1" max="100" id="retention-count" bind:value={retentionCount} on:input={handleChange}>
            <div class="form-description">Number of backup versions to keep</div>
        </div>
        <div class="form-group">
            <button class="button button-primary" on:click={createManualBackup}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7,10 12,15 17,10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
                Create Manual Backup
            </button>
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.paper} title="Export / Import" description="Move your vault data in or out of the application.">
        <div class="form-group">
            <label class="form-label" for="export-encrypted-select">Export Encrypted</label>
            <div class="input-group">
                <Select bind:value={exportFormat} options={[{value: 'json', label: 'Encrypted JSON'}, {value: 'pulsar', label: '.pulsar Format'}]} ariaLabel="Select export format" />
                <button class="button button-primary" on:click={exportEncrypted}>Export</button>
            </div>
        </div>
        <div class="form-group">
            <div class="toggle-container">
                <div class="toggle-info">
                    <label class="form-label" for="enable-plaintext-export-switch">Allow Plaintext Export</label>
                    <div class="form-description">This is a high-risk action</div>
                </div>
                <Switch id="enable-plaintext-export-switch" checked={enablePlaintextExport} ariaLabel="Toggle plaintext export" on:click={() => handleSwitchChange('enablePlaintextExport')} />
            </div>
             <div>
             {#if enablePlaintextExport}
                <button class="button button-warning" style="margin-top: 1rem;" on:click={exportPlaintext}>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                    Export Plaintext
                </button>
            {/if}
            </div>
        </div>
        <div class="form-group">
            <button class="button button-secondary" on:click={importData} style="width: 100%;">Import Data</button>
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.globe} title="Sync" description="Configure automatic synchronization across devices.">
        <div class="form-group">
            <div class="form-label">Sync Mode</div>
            <div class="radio-group">
                <button role="radio" aria-checked={syncMode === 'off'} aria-label="Set sync mode to off" class="radio-option" class:selected={syncMode === 'off'} on:click={() => handleSyncModeChange('off')}>
                    <div class="radio-button"></div>
                    <div>
                        <div class="form-label">Off</div>
                        <div class="form-description">No synchronization</div>
                    </div>
                </button>
                <button role="radio" aria-checked={syncMode === 'manual'} aria-label="Set sync mode to manual" class="radio-option" class:selected={syncMode === 'manual'} on:click={() => handleSyncModeChange('manual')}>
                    <div class="radio-button"></div>
                    <div>
                        <div class="form-label">Manual</div>
                        <div class="form-description">Sync only when triggered</div>
                    </div>
                </button>
                <button role="radio" aria-checked={syncMode === 'auto'} aria-label="Set sync mode to automatic" class="radio-option" class:selected={syncMode === 'auto'} on:click={() => handleSyncModeChange('auto')}>
                    <div class="radio-button"></div>
                    <div>
                        <div class="form-label">Automatic</div>
                        <div class="form-description">Sync when changes are detected</div>
                    </div>
                </button>
            </div>
        </div>
        <div class="form-group">
            <div class="form-label">Storage Providers</div>
            <div class="grid grid-2">
                <button class="provider-card" aria-label="Connect to WebDAV" class:selected={selectedProvider === 'webdav'} on:click={() => openProviderModal('webdav')}>
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/></svg>
                <div>
                    <h4>WebDAV</h4>
                    <span class="status-badge status-disconnected">Not connected</span>
                </div>
                </button>
            <button class="provider-card" aria-label="Connect to Dropbox" class:selected={selectedProvider === 'dropbox'} on:click={() => openProviderModal('dropbox')}>
                 <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="m12 12 5.5-4.5-5.5-4.5-5.5 4.5 5.5 4.5zM6.5 12l5.5 4.5-5.5 4.5-5.5-4.5L6.5 12zm11 0L12 16.5l5.5 4.5 5.5-4.5-5.5-4.5z"/></svg>
                <div>
                    <h4>Dropbox</h4>
                    <span class="status-badge status-disconnected">Not connected</span>
                </div>
            </button>
            <button class="provider-card" aria-label="Connect to Amazon S3" class:selected={selectedProvider === 's3'} on:click={() => openProviderModal('s3')}>
                 <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
                <div>
                    <h4>Amazon S3</h4>
                    <span class="status-badge status-disconnected">Not connected</span>
                </div>
            </button>
            <button class="provider-card" aria-label="Connect to custom storage provider" class:selected={selectedProvider === 'custom'} on:click={() => openProviderModal('custom')}>
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 17.58A5 5 0 0 0 18 8h-1.26A8 8 0 1 0 4 16.25"/></svg>
                <div>
                    <h4>Custom</h4>
                    <span class="status-badge status-warning">Setup Required</span>
                </div>
            </button>
        </div>
        </div>
    </SettingsCard>
</div>

{#if showModal}
    <div class="modal-backdrop" role="button" tabindex="0" aria-label="Close modal" on:click={() => showModal = false} on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') showModal = false; }} transition:fade={{ duration: 150 }}>
        <div class="modal-content" role="document" tabindex="-1" transition:slide={{ duration: 250, easing: quintOut }}>
            <div class="modal-header">
                <svg class="icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                <h3>{modalContent.title}</h3>
            </div>

            {#if modalContent.type === 'confirm'}
                <p>{modalContent.description}</p>
                <div class="modal-actions">
                    <button class="button button-outline" on:click={() => showModal = false}>Cancel</button>
                    <button class="button button-primary" class:button-warning={modalContent.title.includes('WARNING')} on:click={modalContent.onConfirm}>
                        Confirm
                    </button>
                </div>
            {/if}

            {#if modalContent.type === 'provider'}
                <div class="provider-form">
                    {#if modalContent.provider === 'webdav'}
                        <div class="form-group"><label class="form-label" for="server-url">Server URL</label><input type="url" class="input" placeholder="https://your-server.com/webdav/" id="server-url"></div>
                        <div class="form-group"><label class="form-label" for="username">Username</label><input type="text" class="input" id="username"></div>
                        <div class="form-group"><label class="form-label" for="password">Password</label><input type="password" class="input" id="password"></div>
                    {:else if modalContent.provider === 'dropbox'}
                        <p>You will be redirected to Dropbox to authorize this application.</p>
                        <button class="button button-primary">Connect to Dropbox</button>
                    {:else if modalContent.provider === 's3'}
                        <div class="form-group"><label class="form-label" for="bucket-name">Bucket Name</label><input type="text" class="input" id="bucket-name"></div>
                        <div class="form-group"><label class="form-label" for="access-key-id">Access Key ID</label><input type="text" class="input" id="access-key-id"></div>
                        <div class="form-group"><label class="form-label" for="secret-access-key">Secret Access Key</label><input type="password" class="input" id="secret-access-key"></div>
                    {:else if modalContent.provider === 'custom'}
                        <p>Enter details for your custom sync endpoint.</p>
                        <div class="form-group"><label class="form-label" for="endpoint-url">Endpoint URL</label><input type="url" class="input" id="endpoint-url"></div>
                    {/if}
                </div>
                <div class="modal-actions" style="margin-top: 1.5rem;">
                    <button class="button button-outline" on:click={() => showModal = false}>Cancel</button>
                    <button class="button button-primary" on:click={modalContent.onConfirm}>Save</button>
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    

    * {
        box-sizing: border-box;
    }

    .settings-container {
        flex: 1;
        padding: 32px;
        overflow-y: auto;
        position: relative;
        animation: slideUp 0.5s ease-out;
    }

    

    .form-group {
        margin-bottom: 1.25rem;
    }

    .form-group:last-child {
        margin-bottom: 0;
    }

    .form-label {
        display: block;
        font-size: 0.875rem;
        font-weight: 500;
        margin-bottom: 0.5rem;
        color: hsl(var(--foreground));
    }

    .form-description {
        font-size: 0.75rem;
        color: hsl(var(--muted-foreground));
        margin-top: 0.25rem;
    }

    .input-group {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .toggle-container {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1rem;
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        background: hsl(var(--muted) / 0.3);
    }

    .toggle-info {
        flex: 1;
    }

    

    .input {
        width: 100%;
        padding: 0.75rem;
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        background: hsl(var(--background));
        color: hsl(var(--foreground));
        font-size: 0.875rem;
        transition: var(--transition-smooth);
    }

    .input:focus {
        outline: none;
        border-color: hsl(var(--ring));
        box-shadow: 0 0 0 3px hsl(var(--ring) / 0.2);
    }

    

    .button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.625rem 1.25rem;
        border-radius: var(--radius);
        font-size: 0.875rem;
        font-weight: 500;
        border: none;
        cursor: pointer;
        transition: var(--transition-smooth);
        text-decoration: none;
        gap: 0.5rem;
    }

    .button-primary {
        background: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
    }

    .button-primary:hover {
        background: hsl(var(--primary) / 0.9);
    }

    .button-secondary {
        background: hsl(var(--secondary));
        color: hsl(var(--secondary-foreground));
    }

    .button-secondary:hover {
        background: hsl(var(--secondary) / 0.8);
    }

    .button-outline {
        border: 1px solid hsl(var(--border));
        background: transparent;
        color: hsl(var(--foreground));
    }

    .button-outline:hover {
        background: hsl(var(--muted));
    }

    .button-warning {
        background: hsl(var(--warning));
        color: hsl(var(--warning-foreground));
    }

    .button-warning:hover {
        background: hsl(var(--warning) / 0.9);
    }
    
    .grid {
        display: grid;
        gap: 1rem;
    }

    .grid-2 {
        grid-template-columns: repeat(2, 1fr);
    }

    .status-badge {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.75rem;
        font-weight: 500;
    }

    .status-disconnected {
        background: hsl(var(--destructive) / 0.2);
        color: hsl(var(--destructive));
    }

    .status-warning {
        background: hsl(var(--warning) / 0.2);
        color: hsl(var(--warning));
    }

    .provider-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        padding: 1rem;
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        background: hsl(var(--muted) / 0.3);
        transition: var(--transition-smooth);
        cursor: pointer;
    }

    .provider-card:hover {
        border-color: hsl(var(--primary));
        transform: translateY(-3px);
    }

    .provider-card.selected {
        border-color: hsl(var(--primary));
        background: hsl(var(--primary) / 0.1);
    }

    

    .provider-card h4 {
        font-size: 0.875rem;
        font-weight: 600;
        margin-bottom: 0.5rem;
    }

    .radio-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .radio-option {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem;
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        cursor: pointer;
        transition: var(--transition-smooth);
    }

    .radio-option:hover {
        background: hsl(var(--muted) / 0.5);
    }

    .radio-option.selected {
        border-color: hsl(var(--primary));
        background: hsl(var(--primary) / 0.1);
        border-color: hsl(var(--primary));
    }

    .radio-button {
        width: 16px;
        height: 16px;
        border: 2px solid hsl(var(--border));
        border-radius: 50%;
        position: relative;
        flex-shrink: 0;
    }

    .radio-option.selected .radio-button {
        border-color: hsl(var(--primary));
    }

    .radio-option.selected .radio-button::after {
        content: '';
        position: absolute;
        width: 8px;
        height: 8px;
        background: hsl(var(--primary));
        border-radius: 50%;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }

    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: hsl(var(--background) / 0.8);
        backdrop-filter: blur(8px);
        z-index: 50;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .modal-content {
        background: hsl(var(--card));
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        padding: 1.5rem;
        max-width: 480px;
        width: 90%;
        box-shadow: var(--shadow-elegant);
    }

    .modal-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 0.75rem;
    }

    .modal-header .icon {
        color: hsl(var(--primary));
    }

    .modal-header h3 {
        margin-bottom: 0;
        font-size: 1.125rem;
    }

    .modal-content p {
        margin-bottom: 1.5rem;
        color: hsl(var(--muted-foreground));
        line-height: 1.5;
        font-size: 0.875rem;
    }

    .modal-actions {
        display: flex;
        gap: 0.75rem;
        justify-content: flex-end;
    }
    
    @keyframes slideUp {
        from {
            opacity: 0;
            transform: translateY(20px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    @media (max-width: 768px) {
        .grid-2 {
            grid-template-columns: 1fr;
        }
    }
</style>