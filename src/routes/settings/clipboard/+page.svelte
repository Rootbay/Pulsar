<script lang="ts">
    import { onMount, tick } from 'svelte';
    
    import { slide, fade } from 'svelte/transition';
    import { clipboardSettings } from '$lib/stores/clipboard';
    import Switch from '$lib/components/ui/Switch.svelte';
    import SettingItem from '$lib/components/ui/SettingItem.svelte';
    import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
    import { iconPaths } from '$lib/icons';

    let clipboardIntegration: boolean;
    let clearAfterDuration: number;
    let blockHistory: boolean;
    let onlyUnlocked: boolean;
    let permissionLevel: string;

    clipboardSettings.subscribe(value => {
        clipboardIntegration = value.clipboardIntegration;
        clearAfterDuration = value.clearAfterDuration;
        blockHistory = value.blockHistory;
        onlyUnlocked = value.onlyUnlocked;
        permissionLevel = value.permissionLevel;
    });

    let showAuditLog = false;

    $: timeoutLabel = clearAfterDuration === 12 ? "12 seconds (default)" : `${clearAfterDuration} seconds`;

    const auditLogEntries = [
        { id: 1, action: "Password for gmail.com", time: "2 minutes ago", status: "copied" },
        { id: 2, action: "Username: john.doe@email.com", time: "15 minutes ago", status: "copied" },
        { id: 3, action: "Password for github.com", time: "1 hour ago", status: "copied" },
        { id: 4, action: "API Key: sk-proj-...", time: "3 hours ago", status: "copied" }
    ];

    function handleSetTimeout(seconds: number) {
        clipboardSettings.update(current => ({
            ...current,
            clearAfterDuration: seconds
        }));
    }

    function handleSwitchChange(setting: 'clipboardIntegration' | 'blockHistory' | 'onlyUnlocked') {
        clipboardSettings.update(current => ({
            ...current,
            [setting]: !current[setting]
        }));
    }

    function handleRangeChange() {
        // The bind:value already updates clearAfterDuration, so we just need to update the store
        clipboardSettings.update(current => ({
            ...current,
            clearAfterDuration: clearAfterDuration
        }));
    }

    function handleRadioChange() {
        // The bind:group already updates permissionLevel, so we just need to update the store
        clipboardSettings.update(current => ({
            ...current,
            permissionLevel: permissionLevel
        }));
    }

    async function toggleAuditLog() {
        showAuditLog = !showAuditLog;
        if(showAuditLog) {
            await tick();
            const element = document.getElementById('auditLogSection');
            element?.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
    }

    onMount(() => {
        // No need to register module here, it's done in clipboard.ts
        // No need to initialize savedClipboardSettings here, it's done in clipboard.ts
    });
</script>
    

<div class="settings-main-container">
    <SettingsCard icon={iconPaths.paper} title="Core Integration" description="Configure basic clipboard functionality">

        <SettingItem>
            <div slot="info" class="settings-item-content">
                <div class="settings-icon primary">
                    <svg class="icon" viewBox="0 0 24 24"><rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/></svg>
                </div>
                <div class="settings-text">
                    <h4>Clipboard Integration</h4>
                    <p>Enable automatic clipboard functionality</p>
                </div>
            </div>
            <div slot="control">
                <Switch checked={clipboardIntegration} ariaLabel="Toggle clipboard integration" on:click={() => handleSwitchChange('clipboardIntegration')} />
            </div>
        </SettingItem>

        <div style="flex-direction: column; align-items: flex-start;">
            <SettingItem>
                <div slot="info" class="settings-item-content">
                    <div class="settings-icon primary">
                        <svg class="icon" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/></svg>
                    </div>
                    <div class="settings-text">
                        <h4>Clear clipboard after</h4>
                        <p>Automatically clear sensitive data</p>
                    </div>
                </div>
            </SettingItem>

            <div class="timeout-controls">
                <div class="timeout-header">
                    <span>{timeoutLabel}</span>
                    <div class="timeout-buttons">
                        <button class="btn" class:active={clearAfterDuration === 5} on:click={() => handleSetTimeout(5)}>5s</button>
                        <button class="btn" class:active={clearAfterDuration === 12} on:click={() => handleSetTimeout(12)}>12s</button>
                        <button class="btn" class:active={clearAfterDuration === 30} on:click={() => handleSetTimeout(30)}>30s</button>
                        <button class="btn" class:active={clearAfterDuration === 60} on:click={() => handleSetTimeout(60)}>60s</button>
                    </div>
                </div>

                <div class="slider-container">
                    <input type="range" class="range-slider" min="5" max="120" bind:value={clearAfterDuration} on:input={handleRangeChange}>
                    <div class="slider-labels">
                        <span>5s</span>
                        <span>120s</span>
                    </div>
                </div>
            </div>
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.lock} title="Security & Privacy" description="Advanced clipboard security options">
        <SettingItem>
            <div slot="info" class="settings-item-content">
                <div class="settings-icon success">
                    <svg class="icon" viewBox="0 0 24 24"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5M12 7v5l4 2"/></svg>
                </div>
                <div class="settings-text">
                    <h4>Block clipboard history</h4>
                    <p>Prevent system from storing clipboard history</p>
                </div>
            </div>
            <div slot="control">
                <Switch checked={blockHistory} ariaLabel="Toggle block clipboard history" on:click={() => handleSwitchChange('blockHistory')} />
            </div>
        </SettingItem>

        <SettingItem>
            <div slot="info" class="settings-item-content">
                <div class="settings-icon warning">
                    <svg class="icon" viewBox="0 0 24 24"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                </div>
                <div class="settings-text">
                    <h4>Only allow on unlocked session</h4>
                    <p>Disable clipboard when app is locked</p>
                </div>
            </div>
            <div slot="control">
                <Switch checked={onlyUnlocked} ariaLabel="Toggle only allow on unlocked session" on:click={() => handleSwitchChange('onlyUnlocked')} />
            </div>
        </SettingItem>

        <div style="flex-direction: column; align-items: flex-start;">
            <SettingItem>
                <div slot="info" class="settings-item-content">
                    <div class="settings-icon primary">
                        <svg class="icon" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10"/></svg>
                    </div>
                    <div class="settings-text">
                        <h4>Per-item clipboard permissions</h4>
                        <p>Choose permission level when copying items</p>
                    </div>
                </div>
                <div slot="control">
                    <!-- No control here, it's below -->
                </div>
            </SettingItem>

            <div class="radio-group">
                <div class="radio-item">
                    <input type="radio" id="allow-once" name="permission" value="ask" bind:group={permissionLevel} on:change={handleRadioChange}>
                    <label for="allow-once">Allow once (ask each time)</label>
                </div>
                <div class="radio-item">
                    <input type="radio" id="allow-always" name="permission" value="remember" bind:group={permissionLevel} on:change={handleRadioChange}>
                    <label for="allow-always">Always allow (remember choice)</label>
                </div>
            </div>
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.eye} title="Actions & Monitoring" description="Manual controls and activity tracking">
        <div class="action-buttons">
            <button class="btn destructive">
                <svg class="icon-sm" viewBox="0 0 24 24"><path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2M10 11v6M14 11v6"/></svg>
                <span>Clear Clipboard Now</span>
            </button>
            <button class="btn" on:click={toggleAuditLog}>
                <svg class="icon-sm" viewBox="0 0 24 24"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                <span>{showAuditLog ? 'Hide' : 'View'} Audit Log</span>
            </button>
        </div>

        {#if showAuditLog}
            <div id="auditLogSection" class="audit-log" transition:slide>
                <div class="separator"></div>
                <h4 style="margin-bottom: 0.5rem;">Recent Clipboard Activity</h4>
                <p style="color: hsl(var(--muted-foreground)); font-size: 0.875rem; margin-bottom: 1rem;">Local activity log (not synced)</p>

                <div style="max-height: 16rem; overflow-y: auto;">
                    {#each auditLogEntries as entry (entry.id)}
                        <div class="audit-entry" in:fade={{delay: entry.id * 50}}>
                            <div class="audit-content">
                                <svg class="icon-sm" viewBox="0 0 24 24" style="color: hsl(var(--success));"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                                <div class="audit-text">
                                    <p>{entry.action}</p>
                                    <p>{entry.time}</p>
                                </div>
                            </div>
                            <div class="audit-badge">{entry.status}</div>
                        </div>
                    {/each}
                </div>
            </div>
        {/if}
    </SettingsCard>
</div>

<style>
    * {
        box-sizing: border-box;
    }

    .settings-main-container {
        flex: 1;
        padding: 32px;
        overflow-y: auto;
        position: relative;
    }

    .settings-item-content {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .settings-icon {
        width: 40px;
        height: 40px;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
    }

    .settings-icon.primary {
        background: hsl(var(--primary) / 0.1);
        color: hsl(var(--primary));
    }

    .settings-icon.success {
        background: hsl(var(--success) / 0.1);
        color: hsl(var(--success));
    }

    .settings-icon.warning {
        background: hsl(var(--warning) / 0.1);
        color: hsl(var(--warning));
    }

    .settings-text h4 {
        font-weight: 500;
        margin-bottom: 0.25rem;
    }

    .settings-text p {
        font-size: 0.875rem;
        color: hsl(var(--muted-foreground));
    }

    .timeout-controls {
        margin-left: 3.25rem;
        margin-top: 1rem;
        width: calc(100% - 3.25rem);
    }

    .timeout-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 1rem;
        flex-wrap: wrap;
        gap: 1rem;
    }

    .timeout-buttons {
        display: flex;
        gap: 0.5rem;
    }

    .btn {
        padding: 0.5rem 0.75rem;
        border-radius: 6px;
        border: 1px solid hsl(var(--border));
        background: transparent;
        color: hsl(var(--foreground));
        cursor: pointer;
        font-size: 0.875rem;
        transition: var(--transition-smooth);
        display: inline-flex;
        align-items: center;
        gap: 0.25rem;
    }

    .btn:hover {
        background: hsl(var(--muted));
    }

    .btn.active {
        background: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
        border-color: hsl(var(--primary));
    }

    .btn.destructive {
        background: hsl(var(--destructive));
        color: hsl(var(--destructive-foreground));
        border-color: hsl(var(--destructive));
    }

    .btn.destructive:hover {
        background: hsl(var(--destructive) / 0.9);
    }

    .slider-container {
        margin: 1rem 0;
    }

    .range-slider {
        width: 100%;
        height: 6px;
        border-radius: 3px;
        background: hsl(var(--muted));
        outline: none;
        appearance: none;
        -webkit-appearance: none;
    }

    .range-slider::-webkit-slider-thumb {
        appearance: none;
        -webkit-appearance: none;
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: hsl(var(--primary));
        cursor: pointer;
    }

    .range-slider::-moz-range-thumb {
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: hsl(var(--primary));
        cursor: pointer;
        border: none;
    }

    .slider-labels {
        display: flex;
        justify-content: space-between;
        margin-top: 0.5rem;
        font-size: 0.75rem;
        color: hsl(var(--muted-foreground));
    }

    .radio-group {
        margin-left: 3.25rem;
        margin-top: 0.75rem;
    }

    .radio-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
    }

    .radio-item input[type="radio"] {
        accent-color: hsl(var(--primary));
    }

    .radio-item label {
        font-size: 0.875rem;
        cursor: pointer;
    }

    .action-buttons {
        display: flex;
        flex-wrap: wrap;
        gap: 1rem;
        margin-bottom: 1.5rem;
    }

    .audit-log {
        animation: scaleIn 0.3s ease-out;
        margin-top: 1.5rem;
    }

    .audit-entry {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.75rem;
        border-radius: 8px;
        background: hsl(var(--muted) / 0.1);
        border: 1px solid hsl(var(--border) / 0.2);
        margin-bottom: 0.5rem;
    }
    
    .audit-content {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .audit-text p:first-child {
        font-weight: 500;
        font-size: 0.875rem;
        margin-bottom: 0.25rem;
    }

    .audit-text p:last-child {
        font-size: 0.75rem;
        color: hsl(var(--muted-foreground));
    }

    .audit-badge {
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.75rem;
        background: transparent;
        border: 1px solid hsl(var(--border));
        color: hsl(var(--foreground));
    }

    .separator {
        height: 1px;
        background: hsl(var(--border));
        margin: 1.5rem 0;
    }

    .icon {
        width: 20px;
        height: 20px;
        fill: none;
        stroke: currentColor;
        stroke-width: 2;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    .icon-sm {
        width: 16px;
        height: 16px;
    }

    @keyframes scaleIn {
        from {
            opacity: 0;
            transform: scale(0.95);
        }
        to {
            opacity: 1;
            transform: scale(1);
        }
    }

    @keyframes slideUp {
        from {
            opacity: 0;
            transform: translateY(30px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
</style>