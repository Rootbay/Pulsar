<script lang="ts">
    
    import { advancedSettings } from '$lib/stores/advanced'; // Import the new store
    import Switch from '$lib/components/ui/Switch.svelte';
    import SettingItem from '$lib/components/ui/SettingItem.svelte';
    import SettingsCard from '$lib/components/ui/SettingsCard.svelte';

    // No need to define interface or default settings here anymore
    // No need for writable, derived, or deepEqual here anymore

    let kdfPreset: 'fast' | 'balanced' | 'secure' | 'paranoid';
    let timeCost: number;
    let memoryCost: number;
    let parallelism: number;
    let wipeConfirmationText: string;
    let lockMemoryPages: boolean;
    let secureMemoryAllocation: boolean;

    // Subscribe to the advancedSettings store
    advancedSettings.subscribe(value => {
        kdfPreset = value.kdfPreset;
        timeCost = value.timeCost;
        memoryCost = value.memoryCost;
        parallelism = value.parallelism;
        wipeConfirmationText = value.wipeConfirmationText;
        lockMemoryPages = value.lockMemoryPages;
        secureMemoryAllocation = value.secureMemoryAllocation;
    });

    function updateSettings(newValues: Partial<typeof $advancedSettings>) {
        advancedSettings.update(current => ({
            ...current,
            ...newValues
        }));
    }

    function setKDFPreset(preset: 'fast' | 'balanced' | 'secure' | 'paranoid') {
        const presets = {
            fast: { time: 1, memory: 16, parallel: 1 },
            balanced: { time: 3, memory: 64, parallel: 4 },
            secure: { time: 6, memory: 256, parallel: 8 },
            paranoid: { time: 12, memory: 1024, parallel: 16 }
        };
        const config = presets[preset];
        updateSettings({
            kdfPreset: preset,
            timeCost: config.time,
            memoryCost: config.memory,
            parallelism: config.parallel
        });
    }

    function handleChange() {
        updateSettings({
            kdfPreset,
            timeCost,
            memoryCost,
            parallelism,
            wipeConfirmationText,
            lockMemoryPages,
            secureMemoryAllocation
        });
    }

    function handleSwitchChange(setting: 'lockMemoryPages' | 'secureMemoryAllocation') {
        if (setting === 'lockMemoryPages') {
            updateSettings({ lockMemoryPages: !lockMemoryPages });
        } else if (setting === 'secureMemoryAllocation') {
            updateSettings({ secureMemoryAllocation: !secureMemoryAllocation });
        }
    }

    // No need for save() and reset() here, as they are handled by secureStorage.createStore
    // No need for onMount to register module, as secureStorage.createStore does it

</script>

<div class="advanced-container">
    <SettingsCard icon="<path d='M19.14 12.94a4 4 0 0 0-5.66 0l-4.28 4.28a2 2 0 0 0 0 2.83l1.42 1.42a2 2 0 0 0 2.83 0l4.28-4.28a4 4 0 0 0 0-5.66z'/><path d='M14.83 4.83a4 4 0 0 0-5.66 0l-4.28 4.28a2 2 0 0 0 0 2.83l1.42 1.42a2 2 0 0 0 2.83 0l4.28-4.28a4 4 0 0 0 0-5.66z'/>" title="KDF Tuning (Argon2id)" description="Configure key derivation function parameters.">

        <div class="alert alert-warning">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
            <span>Higher values are more secure but will slow down authentication.</span>
        </div>

        <fieldset class="form-group">
            <legend class="label">Presets</legend>
            <div class="preset-buttons">
                <button class="preset-button" class:active={kdfPreset === 'fast'} on:click={() => setKDFPreset('fast')}>Fast</button>
                <button class:active={kdfPreset === 'balanced'} on:click={() => setKDFPreset('balanced')}>Balanced</button>
                <button class:active={kdfPreset === 'secure'} on:click={() => setKDFPreset('secure')}>Secure</button>
                <button class:active={kdfPreset === 'paranoid'} on:click={() => setKDFPreset('paranoid')}>Paranoid</button>
            </div>
        </fieldset>
        <div class="form-group">
            <label class="label" for="time-cost">Time Cost (iterations)</label>
            <div class="slider-container">
                <input type="range" class="slider" min="1" max="20" bind:value={timeCost} on:input={handleChange} id="time-cost">
                <span class="slider-value">{timeCost}</span>
            </div>
        </div>
        <div class="form-group">
            <label class="label" for="memory-cost">Memory Cost</label>
            <div class="slider-container">
                <input type="range" class="slider" min="16" max="1024" step="16" bind:value={memoryCost} on:input={handleChange} id="memory-cost">
                <span class="slider-value">{memoryCost} MB</span>
            </div>
        </div>
        <div class="form-group">
            <label class="label" for="parallelism">Parallelism</label>
            <div class="slider-container">
                <input type="range" class="slider" min="1" max="16" bind:value={parallelism} on:input={handleChange} id="parallelism">
                <span class="slider-value">{parallelism} threads</span>
            </div>
        </div>
    </SettingsCard>

    <SettingsCard icon="<path d='M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z'/><path d='m9 12 2 2 4-4'/>" title="Memory Hardening" description="Protect sensitive data in system memory.">
        <div class="settings-group">
            <SettingItem>
                 <div slot="info" class="setting-info">
                    <div class="setting-icon"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg></div>
                    <div>
                        <label class="label" for="lock-memory-pages-switch" style="margin-bottom: 0;">Lock Memory Pages</label>
                        <div class="description">Prevent data from being swapped to disk</div>
                    </div>
                </div>
                 <div slot="control">
                    <Switch checked={lockMemoryPages} ariaLabel="Toggle lock memory pages" on:click={() => handleSwitchChange('lockMemoryPages')} />
                </div>
            </SettingItem>
            <SettingItem>
                 <div slot="info" class="setting-info">
                    <div class="setting-icon"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19.4 19.4a2 2 0 0 1-2.8 0L12 14.8l-4.6 4.6a2 2 0 0 1-2.8-2.8l4.6-4.6-4.6-4.6a2 2 0 0 1 2.8-2.8l4.6 4.6 4.6-4.6a2 2 0 0 1 2.8 2.8L14.8 12l4.6 4.6a2 2 0 0 1 0 2.8z"/></svg></div>
                    <div>
                        <div class="label" style="margin-bottom: 0;">Secure Memory Allocation</div>
                        <div class="description">Use secure allocator for sensitive data</div>
                    </div>
                </div>
                 <div slot="control">
                    <Switch checked={secureMemoryAllocation} ariaLabel="Toggle secure memory allocation" on:click={() => handleSwitchChange('secureMemoryAllocation')} />
                </div>
            </SettingItem>
        </div>
    </SettingsCard>

    <SettingsCard icon="<path d='M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z'/>" title="Destructive Actions" description="These actions are irreversible and will delete data.">
        
        <div class="danger-zone">
            <div class="label danger-title">Wipe Vault Database</div>
            <p class="description" style="margin-bottom: 1rem;">Permanently delete all vault data and settings.</p>
            <input type="text" class="input is-danger" placeholder='Type "DELETE VAULT" to confirm' bind:value={wipeConfirmationText} on:input={handleChange}>
            <button class="button button-destructive" disabled={wipeConfirmationText !== 'DELETE VAULT'} style="margin-top: 0.75rem; width: 100%;">
                Wipe Vault Database
            </button>
        </div>
    </SettingsCard>
</div>

<style>
    * {
        box-sizing: border-box;
    }

    .advanced-container {
        flex: 1;
        padding: 32px;
        overflow-y: auto;
        position: relative;
        animation: slideUp 0.5s ease-out;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .form-group {
        margin-bottom: 1.5rem;
    }

    .form-group:last-child {
        margin-bottom: 0;
    }

    fieldset {
        border: none;
        padding: 0;
        margin: 0;
    }

    .label {
        display: block;
        font-weight: 500;
        margin-bottom: 0.5rem;
        color: hsl(var(--foreground));
    }

    .description {
        color: hsl(var(--muted-foreground));
        font-size: 0.875rem;
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

    .input.is-danger {
        border-color: hsl(var(--destructive));
    }

    .input.is-danger:focus {
        box-shadow: 0 0 0 3px hsl(var(--destructive) / 0.3);
    }

    .slider-container {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .slider {
        flex: 1;
        height: 6px;
        border-radius: 3px;
        background: hsl(var(--secondary));
        outline: none;
        -webkit-appearance: none;
        appearance: none;
    }

    .slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 20px;
        height: 20px;
        border-radius: 50%;
        background: hsl(var(--primary));
        cursor: pointer;
        border: 2px solid hsl(var(--foreground));
    }

    .slider-value {
        min-width: 4.5rem;
        text-align: right;
        color: hsl(var(--muted-foreground));
        font-size: 0.875rem;
    }

    .preset-buttons {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 1rem;
    }

    .preset-button {
        padding: 0.25rem 0.75rem;
        background: hsl(var(--muted));
        border: 1px solid hsl(var(--border));
        border-radius: 99px;
        cursor: pointer;
        font-size: 0.75rem;
        transition: all 0.2s;
        color: hsl(var(--muted-foreground));
    }

    .preset-button:hover {
        background: hsl(var(--secondary));
        color: hsl(var(--foreground));
    }

    .preset-button.active {
        background: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
        border-color: hsl(var(--primary));
    }

    .alert {
        padding: 1rem;
        margin-bottom: 1rem;
        border-radius: var(--radius);
        font-size: 0.875rem;
    }

    .alert-warning {
        background: hsl(var(--warning) / 0.1);
        border: 1px solid hsl(var(--warning) / 0.3);
        color: hsl(var(--warning));
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .danger-zone {
        padding: 1.25rem;
        border-radius: var(--radius);
        background: hsl(var(--destructive) / 0.1);
        border: 1px solid hsl(var(--destructive) / 0.3);
    }

    .danger-title {
        color: hsl(var(--destructive));
        font-weight: 600;
        margin-bottom: 0.5rem;
    }

    .settings-group {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    

    .setting-info {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .setting-icon {
        color: hsl(var(--muted-foreground));
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

    .button-destructive {
        background: hsl(var(--destructive));
        color: hsl(var(--primary-foreground));
    }

    .button-destructive:hover {
        background: hsl(var(--destructive) / 0.9);
    }

    .button:disabled {
        background: hsl(var(--muted));
        color: hsl(var(--muted-foreground));
        cursor: not-allowed;
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