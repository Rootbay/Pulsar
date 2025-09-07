<script lang="ts">
    
    import Switch from '$lib/components/ui/Switch.svelte';
    import SettingsCard from '$lib/components/ui/SettingsCard.svelte';

    let updateStatus = 'idle'; 
    let uploadDiagnostics = false;

    function checkForUpdates() {
        if (updateStatus !== 'idle') return;
        updateStatus = 'checking';
        setTimeout(() => {
            updateStatus = 'uptoDate';
            setTimeout(() => {
                updateStatus = 'idle';
            }, 2000);
        }, 1500);
    }

    function openLink(linkName: string) {
        alert(`Opening ${linkName}...`);
    }
</script>

<div class="about-container">
    <SettingsCard icon="<path d='M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z'/>" title="Application Information" description="Version details and update status.">
        <div class="version-grid">
            <div class="version-item"><div class="version-label">Version</div><div class="version-value">2.1.4</div></div>
            <div class="version-item"><div class="version-label">Build</div><div class="version-value">a7f3d2e</div></div>
            <div class="version-item"><div class="version-label">Runtime</div><div class="version-value">Rust 1.75</div></div>
            <div class="version-item"><div class="version-label">UI</div><div class="version-value">Svelte 4.2</div></div>
            <div class="version-item"><div class="version-label">Status</div><div class="version-value status">Up to date</div></div>
        </div>

        <div class="form-group" style="margin-top: 1.5rem; margin-bottom: 0;">
            <div class="setting-item">
                <div class="setting-info">
                    <div>
                        <div class="setting-label">Check for Updates</div>
                        <div class="setting-description">Check if a newer version is available</div>
                    </div>
                </div>
                <div class="setting-control">
                    <button class="button primary" on:click={checkForUpdates} disabled={updateStatus !== 'idle'}>
                        {#if updateStatus === 'idle'}
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M21 21v-5h-5"/></svg>
                            Check Now
                        {:else if updateStatus === 'checking'}
                            <span>Checking...</span>
                        {:else if updateStatus === 'uptoDate'}
                            <span>✅ Up to date</span>
                        {/if}
                    </button>
                </div>
            </div>
        </div>
    </SettingsCard>

    <SettingsCard icon="<path d='M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm1 15h-2v-2h2zm0-4h-2V7h2z'/>" title="Documentation & Support" description="Find helpful resources and get in touch.">
        <div class="links-grid">
                <button class="button" on:click={() => openLink('License')}>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><line x1="10" y1="9" x2="8" y2="9"/></svg>
                    License
                </button>
                <button class="button" on:click={() => openLink('Whitepaper')}>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                    Security Whitepaper
                </button>
                <button class="button" on:click={() => openLink('Support')}>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                    Contact Support
                </button>
                <button class="button" on:click={() => openLink('Source Code')}>
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>
                    Source Code
                </button>
        </div>
    </SettingsCard>

    <SettingsCard icon="<path d='M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z'/><path d='M12 18v-6'/><path d='M9 15h6'/>" title="Diagnostic Tools" description="Tools for troubleshooting and support.">

        <div class="settings-group">
            <div class="setting-item">
                <div class="setting-info">
                    <div>
                        <div class="setting-label">Open Logs Folder</div>
                        <div class="setting-description">Access application log files</div>
                    </div>
                </div>
                <div class="setting-control">
                    <button class="button" on:click={() => openLink('Logs Folder')}>
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2a2 2 0 0 0-1.66-.9H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2z"/></svg>
                        Open
                    </button>
                </div>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <div>
                        <div class="setting-label">Upload Diagnostics to Support</div>
                        <div class="setting-description">Help us identify and fix issues faster</div>
                    </div>
                </div>
                <div class="setting-control">
                    <Switch checked={uploadDiagnostics} ariaLabel="Toggle upload diagnostics" on:click={() => uploadDiagnostics = !uploadDiagnostics} />
                </div>
            </div>
        </div>
    </SettingsCard>
</div>
<style>
    

    * {
        box-sizing: border-box;
    }

    .about-container {
        flex: 1;
        padding: 32px;
        overflow-y: auto;
        position: relative;
        animation: slideUp 0.5s ease-out;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    

    .version-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
        gap: 1.25rem 1rem;
        padding: 1rem;
        border-radius: var(--radius);
        background: hsl(var(--muted) / 0.2);
    }

    .version-item {
        text-align: center;
    }

    .version-label {
        font-size: 0.8125rem;
        color: hsl(var(--muted-foreground));
        margin-bottom: 0.25rem;
    }

    .version-value {
        font-weight: 600;
        font-family: 'SF Mono', 'Courier New', monospace;
        font-size: 0.875rem;
    }

    .version-value.status {
        color: hsl(var(--success));
    }

    .settings-group {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .setting-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: hsl(var(--muted) / 0.3);
        padding: 1rem;
        border-radius: var(--radius);
        border: 1px solid hsl(var(--border));
    }

    .setting-info {
        flex: 1;
    }

    .setting-label {
        font-weight: 500;
    }

    .setting-description {
        color: hsl(var(--muted-foreground));
        font-size: 0.875rem;
        margin-top: 0.125rem;
    }

    .setting-control {
        margin-left: 1rem;
    }

    .button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.625rem 1.25rem;
        border-radius: var(--radius);
        font-size: 0.875rem;
        font-weight: 500;
        border: 1px solid hsl(var(--border));
        background: hsl(var(--secondary));
        color: hsl(var(--foreground));
        cursor: pointer;
        transition: var(--transition-smooth);
        text-decoration: none;
        gap: 0.5rem;
    }

    .button:hover {
        background: hsl(var(--muted));
        border-color: hsl(var(--ring));
    }

    .button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .button.primary {
        background: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
        border-color: hsl(var(--primary));
    }

    .button.primary:hover {
        background: hsl(var(--primary) / 0.9);
    }

    .links-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 0.75rem;
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
</style>