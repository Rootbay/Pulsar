<script lang="ts">
    import { autofillSettings } from '$lib/stores/autofill';
    import type { AutofillSettings } from '$lib/config/settings';
    import Switch from '$lib/components/ui/Switch.svelte';
    import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
    import { iconPaths } from '$lib/icons';

    let browserAutofill: boolean;
    let globalAutotype: boolean;
    let osUnlock: boolean;
    let perSiteConfirmation: boolean;

    autofillSettings.subscribe(value => {
        browserAutofill = value.browserAutofill;
        globalAutotype = value.globalAutotype;
        osUnlock = value.osUnlock;
        perSiteConfirmation = value.perSiteConfirmation;
    });

    function toggleSetting(key: keyof AutofillSettings) {
        autofillSettings.update(current => ({
            ...current,
            [key]: !current[key]
        }));
    }
</script>

<div class="content-wrapper">
    <SettingsCard icon={iconPaths.eye} title="Browser Auto-fill" description="Configure browser auto-fill settings.">
        <div class="toggle-group">
            <div class="toggle-content">
                <div class="toggle-label">Enable Auto-fill for Browsers</div>
                <div class="toggle-description">Automatically fill login forms in web browsers</div>
            </div>
            <Switch
                checked={browserAutofill}
                ariaLabel="Enable Auto-fill for Browsers"
                on:click={() => toggleSetting('browserAutofill')}
            />
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.key} title="Global Auto-type" description="Configure global auto-type settings.">
        <div class="toggle-group">
            <div class="toggle-content">
                <div class="toggle-label">Enable Global Auto-type</div>
                <div class="toggle-description">Type passwords automatically using keyboard shortcuts</div>
            </div>
            <Switch
                checked={globalAutotype}
                ariaLabel="Enable Global Auto-type"
                on:click={() => toggleSetting('globalAutotype')}
            />
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.security} title="Safety Checks" description="Configure safety checks for autofill.">
        <div class="settings-list">
            <div class="toggle-group">
                <div class="toggle-content">
                    <div class="toggle-label">Require OS-level Unlock for Auto-fill</div>
                    <div class="toggle-description">Require system authentication before auto-filling</div>
                </div>
                <Switch
                    checked={osUnlock}
                    ariaLabel="Require OS-level Unlock for Auto-fill"
                    on:click={() => toggleSetting('osUnlock')}
                />
            </div>
            <div class="toggle-group">
                <div class="toggle-content">
                    <div class="toggle-label">Require Per-site Confirmation</div>
                    <div class="toggle-description">Ask for confirmation before auto-filling on each site</div>
                </div>
                <Switch
                    checked={perSiteConfirmation}
                    ariaLabel="Require Per-site Confirmation"
                    on:click={() => toggleSetting('perSiteConfirmation')}
                />
            </div>
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.edit} title="Test Auto-type" description="Test your auto-type configuration.">
        <p class="toggle-description" style="margin-top: -1.25rem; margin-bottom: 1.25rem;">Test your auto-type configuration to ensure it works correctly with your setup.</p>

        <div class="test-section-header">
            <button class="btn btn-primary">
                <svg class="icon" style="width: 1rem; height: 1rem;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                Simulate Auto-type
            </button>
            <div class="test-section-warning">
                <svg class="icon" style="width: 1rem; height: 1rem; color: hsl(var(--warning));" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.46 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
                Make sure you have a text field selected before testing
            </div>
        </div>

        <div class="test-results-box">
            <h3>Test Results</h3>
            <div class="result-item">
                <svg class="icon icon-success" style="width: 1rem; height: 1rem;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                <span>Hotkey registration: Success</span>
            </div>
            <div class="result-item">
                <svg class="icon icon-success" style="width: 1rem; height: 1rem;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                <span>Keystroke simulation: Success</span>
            </div>
            <div class="result-item">
                <svg class="icon icon-failure" style="width: 1rem; height: 1rem;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
                <span>Browser focus detection: Failed</span>
            </div>
        </div>
    </SettingsCard>
</div>

<style>
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }

    .content-wrapper {
        flex: 1;
        padding: 32px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 1.5rem; 
    }

    .settings-list {
        display: flex;
        flex-direction: column;
        gap: 1.5rem; 
    }

    .btn {
        padding: 0.625rem 1rem; 
        border-radius: var(--radius);
        border: none;
        font-size: 0.875rem; 
        cursor: pointer;
        transition: var(--transition-smooth);
        font-weight: 500;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem; 
    }

    .btn-primary {
        background: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
    }

    .btn-primary:hover {
        background: hsl(var(--primary) / 0.9);
    }

    .toggle-group {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .toggle-content .toggle-label {
        font-size: 0.9375rem; 
        color: hsl(var(--foreground));
        font-weight: 500;
    }

    .toggle-content .toggle-description {
        font-size: 0.875rem; 
        color: hsl(var(--muted-foreground));
        margin-top: 0.25rem; 
    }

    

    .test-section-header {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .test-section-warning {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: hsl(var(--muted-foreground));
        font-size: 0.875rem;
    }

    .test-results-box {
        background: hsl(var(--background));
        padding: 1rem;
        border-radius: var(--radius);
        margin-top: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .test-results-box h3 {
        font-size: 0.875rem;
        font-weight: 600;
        margin-bottom: 0.25rem;
    }

    .result-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-size: 0.875rem;
        color: hsl(var(--foreground));
    }

    .result-item .icon-success {
        color: hsl(var(--success));
    }

    .result-item .icon-failure {
        color: hsl(var(--destructive));
    }

    .icon {
        width: 1.25rem;
        height: 1.25rem;
        stroke-width: 1.5;
        flex-shrink: 0;
    }
</style>