<script lang="ts">
    import { onMount } from 'svelte';
    import { generatorSettings } from '$lib/stores/generator';
    import Switch from '$lib/components/ui/Switch.svelte';
    import Select from '$lib/components/ui/Select.svelte';
    import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
    import { iconPaths } from '$lib/icons';
    import { passwordPresets } from '$lib/stores/passwordPresets';
    import { siteRules } from '$lib/stores/siteRules';
    import EditModal from '$lib/components/ui/EditModal.svelte';
    import type { GeneratorSettings } from '$lib/config/settings';

    type GeneratorOptions = GeneratorSettings['options'];

    // Constants
    const SYMBOL_CHARSET = '!@#$%^&*()_+-=[]{}|;:,.<>?';
    const AMBIGUOUS_CHARS = new Set('iI1LoO0');
    const SIMILAR_CHARS = new Set('oO0l1IvVwWsScCpPkKxXzZbBdDgGqQeEfFtTuUjJmMnnrRhHaAyY');

    const ENTROPY_WEAK_THRESHOLD = 60;
    const ENTROPY_GOOD_THRESHOLD = 120;
    const MAX_ENTROPY_BITS = 128;
    const DEFAULT_PASSWORD_LENGTH = 20;

    const STRENGTH_INFO = {
        weak: { label: 'Weak', color: 'hsl(var(--destructive))' },
        good: { label: 'Good', color: 'hsl(var(--warning))' },
        veryStrong: { label: 'Very Strong', color: 'hsl(var(--success))' }
    };

    let generatedPassword = '';
    let copyButtonText = 'Copy';

    let showEditModal = false;
    let itemToEdit: any | null = null;
    let editModalType: 'preset' | 'rule' | null = null;

    $: ({ passwordLength, options } = $generatorSettings);
    $: passwordLengthString = passwordLength.toString();
    

    function calculateStrength(length: number, opts: GeneratorOptions) {
        let pool = 0;
        if (opts.uppercase) pool += 26;
        if (opts.lowercase) pool += 26;
        if (opts.digits) pool += 10;
        if (opts.symbols) pool += SYMBOL_CHARSET.length; 

        if (pool === 0) {
             return { ...STRENGTH_INFO.weak, entropy: 0 };
        }

        const entropy = Math.round(length * Math.log2(pool));

        if (entropy < ENTROPY_WEAK_THRESHOLD) return { ...STRENGTH_INFO.weak, entropy: entropy };
        if (entropy < ENTROPY_GOOD_THRESHOLD) return { ...STRENGTH_INFO.good, entropy: entropy };
        return { ...STRENGTH_INFO.veryStrong, entropy: entropy };
    }

    $: strengthInfo = calculateStrength(passwordLength, options);

    function buildBaseCharset(options: GeneratorOptions): string {
        let baseCharset = '';
        // Only add characters if at least one option is selected
        if (options.uppercase) baseCharset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
        if (options.lowercase) baseCharset += 'abcdefghijklmnopqrstuvwxyz';
        if (options.digits) baseCharset += '0123456789';
        if (options.symbols) baseCharset += SYMBOL_CHARSET;
        return baseCharset;
    }

    function getEffectiveCharsetArray(baseCharset: string, options: GeneratorOptions): string[] {
        let effectiveCharsetArray = baseCharset.split('');

        if (options.ambiguous) {
            effectiveCharsetArray = effectiveCharsetArray.filter(char => !AMBIGUOUS_CHARS.has(char));
        }

        if (options.similar) {
            effectiveCharsetArray = effectiveCharsetArray.filter(char => !SIMILAR_CHARS.has(char));
        }
        return effectiveCharsetArray;
    }

    function generatePronounceablePassword(length: number, effectiveCharsetArray: string[]): string {
        const vowels = new Set('aeiouAEIOU');
        const consonants = new Set('bcdfghjklmnpqrstvwxyzBCDFGHJKLMNPQRSTVWXYZ');

        const availableVowels = effectiveCharsetArray.filter(char => vowels.has(char));
        const availableConsonants = effectiveCharsetArray.filter(char => consonants.has(char));

        const passwordChars: string[] = [];
        let useVowel = Math.random() < 0.5;

        for (let i = 0; i < length; i++) {
            let charToAdd: string | undefined;
            if (useVowel && availableVowels.length > 0) {
                charToAdd = availableVowels[Math.floor(Math.random() * availableVowels.length)];
            } else if (!useVowel && availableConsonants.length > 0) {
                charToAdd = availableConsonants[Math.floor(Math.random() * availableConsonants.length)];
            }

            if (charToAdd) {
                passwordChars.push(charToAdd);
            } else {
                // Fallback if no suitable char found (e.g., no vowels available)
                passwordChars.push(effectiveCharsetArray[Math.floor(Math.random() * effectiveCharsetArray.length)]);
            }
            useVowel = !useVowel;
        }
        return passwordChars.join('');
    }

    function generateRandomPassword(length: number, effectiveCharsetArray: string[]): string {
        const passwordChars: string[] = [];
        const n = effectiveCharsetArray.length;
        for (let i = 0; i < length; ++i) {
            passwordChars.push(effectiveCharsetArray[Math.floor(Math.random() * n)]);
        }
        return passwordChars.join('');
    }

    function generatePassword() {
        const baseCharset = buildBaseCharset(options);
        const effectiveCharsetArray = getEffectiveCharsetArray(baseCharset, options);

        if (effectiveCharsetArray.length === 0) {
            generatedPassword = '';
            return;
        }

        if (options.pronounceable) {
            generatedPassword = generatePronounceablePassword(passwordLength, effectiveCharsetArray);
        } else {
            generatedPassword = generateRandomPassword(passwordLength, effectiveCharsetArray);
        }
    }

    onMount(() => {
        generatorSettings.update(current => ({
            ...current,
            options: {
                ...current.options,
                lowercase: true
            }
        }));
        generatePassword();
    });

    async function copyPassword() {
        if (!generatedPassword) return;
        try {
            await navigator.clipboard.writeText(generatedPassword);
            copyButtonText = 'Copied!';
            setTimeout(() => { copyButtonText = 'Copy'; }, 2000);
        } catch (err) {
            console.error('Failed to copy: ', err);
        }
    }

    function applyPreset(preset: any) {
        generatorSettings.update(current => ({
            ...current,
            passwordLength: preset.length,
            options: {
                ...current.options,
                uppercase: preset.settings.uppercase,
                lowercase: preset.settings.lowercase,
                digits: preset.settings.digits,
                symbols: preset.settings.symbols,
                ambiguous: preset.settings.ambiguous,
                similar: preset.settings.similar,
                pronounceable: preset.settings.pronounceable,
            }
        }));
        generatePassword();
    }

    function handleGeneratorOptionChange() {
        passwordLength = parseInt(passwordLengthString);
        generatePassword();
    }

    function saveCurrentSettingsAsPreset() {
        const presetName = prompt('Enter a name for this preset:');
        if (presetName) {
            passwordPresets.addPreset({
                name: presetName,
                length: passwordLength,
                charSet: 'Custom', // This could be more dynamic based on selected options
                strength: strengthInfo.entropy,
                settings: { ...options }
            });
        }
    }

    function resetOptions() {
        generatorSettings.update(current => ({
            ...current,
            passwordLength: DEFAULT_PASSWORD_LENGTH, // Default length
            options: {
                uppercase: true,
                lowercase: true,
                digits: true,
                symbols: true,
                ambiguous: false,
                similar: false,
                pronounceable: false,
            }
        }));
        generatePassword();
    }

    function removePreset(name: string) {
        if (confirm(`Are you sure you want to delete preset "${name}"?`)) {
            passwordPresets.deletePreset(name);
        }
    }

    function removeRule(url: string) {
        if (confirm(`Are you sure you want to delete rule for "${url}"?`)) {
            siteRules.deleteRule(url);
        }
    }

    function handleEditPreset(preset: any) {
        itemToEdit = preset;
        editModalType = 'preset';
        showEditModal = true;
    }

    function handleEditRule(rule: any) {
        itemToEdit = rule;
        editModalType = 'rule';
        showEditModal = true;
    }

    function handleSaveEdit(updatedItem: any) {
        if (editModalType === 'preset') {
            passwordPresets.updatePreset(updatedItem.name, updatedItem);
        } else if (editModalType === 'rule') {
            siteRules.updateRule(updatedItem.url, updatedItem);
        }
        itemToEdit = null;
        editModalType = null;
    }
</script>

<div class="content-wrapper">
    <SettingsCard icon={iconPaths.key} title="Password Generator" description="Generate strong and secure passwords.">

        <div class="label">Generated Password</div>
        <div class="password-display-wrapper">
            <span class="password-text">{generatedPassword}</span>
            <span class="password-meta">{passwordLength} chars</span>
            <button class="btn btn-secondary" on:click={generatePassword}><svg viewBox="0 0 24 24" class="icon"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M21 21v-5h-5"/></svg> Generate</button>
            <button class="btn btn-secondary" on:click={copyPassword}><svg viewBox="0 0 24 24" class="icon"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg> {copyButtonText}</button>
        </div>

        <div class="strength-section">
            <div class="strength-header">
                <div class="label" style="margin: 0;">Password Strength</div>
                <div class="strength-value" style="color: {strengthInfo.color};">{strengthInfo.label}</div>
            </div>
            <div class="label" style="font-size: 0.8rem;">Entropy: ~{strengthInfo.entropy} bits</div>
            <div class="strength-progress-bg">
                <div class="strength-progress-fg" style="background-color: {strengthInfo.color}; width: {Math.min(100, (strengthInfo.entropy / 128) * 100)}%;"></div>
            </div>
        </div>

        <div class="generator-grid">
            <div>
                <div class="label">Password Length: {passwordLength}</div>
                <input type="range" min="6" max="64" bind:value={passwordLength} on:input={handleGeneratorOptionChange}>

                <div class="label" style="margin-top: 1.5rem;">Default Preset</div>
                <div class="select-wrapper">
                    <Select bind:value={passwordLengthString} options={$passwordPresets.map(p => ({value: p.length.toString(), label: p.name}))} on:change={handleGeneratorOptionChange} ariaLabel="Select password preset" />
                </div>
            </div>
            <div>
                <div class="options-section-title">Character Options</div>
                <div class="toggle-group" style="margin-bottom: 0.75rem;">
                    <span class="toggle-label">Include uppercase (A-Z)</span>
                    <Switch checked={options.uppercase} ariaLabel="Toggle uppercase characters" on:click={() => {options.uppercase = !options.uppercase; handleGeneratorOptionChange()}}/>
                </div>
                <div class="toggle-group" style="margin-bottom: 0.75rem;">
                    <span class="toggle-label">Include lowercase (a-z)</span>
                    <Switch checked={options.lowercase} ariaLabel="Toggle lowercase characters" on:click={() => {options.lowercase = !options.lowercase; handleGeneratorOptionChange()}}/>
                </div>
                <div class="toggle-group" style="margin-bottom: 0.75rem;">
                    <span class="toggle-label">Include digits (0-9)</span>
                    <Switch checked={options.digits} ariaLabel="Toggle digits" on:click={() => {options.digits = !options.digits; handleGeneratorOptionChange()}}/>
                </div>
                <div class="toggle-group">
                    <span class="toggle-label">Include symbols (!@#$...)</span>
                    <Switch checked={options.symbols} ariaLabel="Toggle symbols" on:click={() => {options.symbols = !options.symbols; handleGeneratorOptionChange()}}/>
                </div>
            </div>
        </div>

        <div class="options-section-title" style="margin-top: 2rem;">Advanced Options</div>
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem;">
             <div class="toggle-group">
                <div>
                    <div class="toggle-label">Avoid ambiguous characters</div>
                    <div class="toggle-desc">Excludes iI1LoO0</div>
                </div>
                <Switch checked={options.ambiguous} ariaLabel="Toggle ambiguous characters" on:click={() => {options.ambiguous = !options.ambiguous; handleGeneratorOptionChange()}}/>
            </div>
            <div class="toggle-group">
                <div>
                    <div class="toggle-label">Exclude similar</div>
                    <div class="toggle-desc">Avoid visually similar</div>
                </div>
                <Switch checked={options.similar} ariaLabel="Toggle similar characters" on:click={() => {options.similar = !options.similar; handleGeneratorOptionChange()}}/>
            </div>
            <div class="toggle-group">
                <div>
                    <div class="toggle-label">Pronounceable mode</div>
                    <div class="toggle-desc">Easier to remember</div>
                </div>
                <Switch checked={options.pronounceable} ariaLabel="Toggle pronounceable mode" on:click={() => {options.pronounceable = !options.pronounceable; handleGeneratorOptionChange()}}/>
            </div>
        </div>

        <div class="action-buttons">
            <button class="btn btn-primary" style="flex-grow: 1;">Generate & Fill</button>
            <button class="btn btn-secondary" style="flex-grow: 1;" on:click={copyPassword}>Generate & Copy</button>
            <button class="btn btn-secondary" on:click={saveCurrentSettingsAsPreset}>Save as Preset</button>
            <button class="btn btn-secondary" on:click={resetOptions}>Reset Options</button>
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.folder} title="Saved Presets" description="Manage your saved password generation presets.">
        <div class="presets-grid">
            {#each $passwordPresets as preset (preset.name)}
                <div class="preset-card">
                    <div class="preset-header">
                        <span class="preset-name">{preset.name}</span>
                        <div class="preset-actions">
                            <button class="btn btn-icon" aria-label="Edit preset" on:click={() => handleEditPreset(preset)}><svg viewBox="0 0 24 24" class="icon" style="width: 0.875rem; height: 0.875rem;"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg></button>
                            <button class="btn btn-icon danger" aria-label="Delete preset" on:click={() => removePreset(preset.name)}><svg viewBox="0 0 24 24" class="icon" style="width: 0.875rem; height: 0.875rem;"><path d="M3 6h18m-2 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
                        </div>
                    </div>
                    <div class="preset-info">Length: {preset.length} characters</div>
                    <div class="preset-info">Character set: {preset.charSet}</div>
                    <div class="preset-strength-bar-bg">
                        <div class="preset-strength-bar-fg" style="width: {Math.min(100, (preset.strength / 128) * 100)}%;"></div>
                    </div>
                    <button class="btn btn-primary" style="width: 100%;" on:click={() => applyPreset(preset)}>Use Preset</button>
                </div>
            {/each}
        </div>
    </SettingsCard>

    <SettingsCard icon={iconPaths.paper} title="Site Rule Templates" description="Manage templates for site-specific password generation rules.">
        <div class="rules-list">
            {#each $siteRules as rule (rule.url)}
                <div class="rule-item">
                    <svg viewBox="0 0 24 24" class="icon" style="color: hsl(var(--muted-foreground)); flex-shrink: 0;"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20z"/><path d="M2 12h20"/></svg>
                    <div class="rule-content">
                        <div class="rule-header">
                            <span class="rule-url">{rule.url}</span>
                            <span class="rule-badge">{rule.length} chars</span>
                            <span class="rule-badge">{rule.type}</span>
                        </div>
                        <div class="toggle-desc">{rule.desc}</div>
                    </div>
                     <div class="preset-actions">
                        <button class="btn btn-icon" aria-label="Edit rule" on:click={() => handleEditRule(rule)}><svg viewBox="0 0 24 24" class="icon" style="width: 0.875rem; height: 0.875rem;"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg></button>
                        <button class="btn btn-icon danger" aria-label="Delete rule" on:click={() => removeRule(rule.url)}><svg viewBox="0 0 24 24" class="icon" style="width: 0.875rem; height: 0.875rem;"><path d="M3 6h18m-2 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
                    </div>
                </div>
            {/each}
        </div>
    </SettingsCard>
</div>

{#if showEditModal}
    <EditModal
        show={showEditModal}
        item={itemToEdit}
        type={editModalType!}
        on:close={() => showEditModal = false}
        on:save={handleSaveEdit}
    />
{/if}

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

    

    .label {
        font-size: 0.875rem;
        font-weight: 500;
        color: hsl(var(--muted-foreground));
        margin-bottom: 0.5rem;
    }

    .password-display-wrapper {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background-color: hsl(var(--background));
        padding: 0.5rem 0.5rem 0.5rem 1rem;
        border-radius: var(--radius);
        margin-bottom: 1rem;
    }

    .password-text {
        flex-grow: 1;
        font-family: 'Courier New', Courier, monospace;
        font-size: 1rem;
        color: hsl(var(--foreground));
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .password-meta {
        font-size: 0.8rem;
        color: hsl(var(--muted-foreground));
    }

    .strength-section {
        margin-bottom: 1.5rem;
    }

    .strength-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .strength-value {
        font-weight: 600;
        font-size: 0.9rem;
    }

    .strength-progress-bg {
        width: 100%;
        height: 5px;
        background-color: hsl(var(--secondary));
        border-radius: 99px;
        margin-top: 0.5rem;
    }

    .strength-progress-fg {
        height: 100%;
        border-radius: 99px;
        transition: background-color 0.3s;
    }

    .generator-grid {
        display: grid;
        grid-template-columns: 2fr 1fr;
        gap: 2.5rem;
        margin-top: 1rem;
    }

    .options-section-title {
        font-size: 0.9rem;
        font-weight: 600;
        margin-bottom: 1rem;
        color: hsl(var(--foreground));
    }

    .action-buttons {
        display: flex;
        gap: 0.75rem;
        flex-wrap: wrap;
        border-top: 1px solid hsl(var(--border));
        margin-top: 1.5rem;
        padding-top: 1.5rem;
    }

    .presets-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
        gap: 1rem;
    }

    .preset-card {
        background: hsl(var(--muted)/0.3);
        border: 1px solid hsl(var(--border));
        border-radius: var(--radius);
        padding: 1rem;
    }

    .preset-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .preset-name {
        font-weight: 600;
    }

    .preset-actions {
        display: flex;
        gap: 0.5rem;
    }

    .preset-info {
        font-size: 0.8rem;
        color: hsl(var(--muted-foreground));
        margin-bottom: 0.25rem;
    }

    .preset-strength-bar-bg {
        width: 100%;
        background-color: hsl(var(--background));
        border-radius: 99px;
        height: 6px;
        margin-top: 1rem;
        margin-bottom: 1rem;
    }

    .preset-strength-bar-fg {
        height: 100%;
        background-color: hsl(var(--primary));
        border-radius: 99px;
    }

    .rules-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .rule-item {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 1rem;
        background: hsl(var(--muted)/0.3);
        border-radius: var(--radius);
        border: 1px solid transparent;
        transition: var(--transition-smooth);
    }

    .rule-item:hover {
        border-color: hsl(var(--border));
    }

    .rule-content {
        flex-grow: 1;
    }

    .rule-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 0.25rem;
    }

    .rule-url {
        font-weight: 600;
    }

    .rule-badge {
        font-size: 0.7rem;
        background: hsl(var(--secondary));
        padding: 0.125rem 0.5rem;
        border-radius: 99px;
    }

    .btn {
        padding: 0.5rem 1rem;
        border-radius: var(--radius);
        border: 1px solid transparent;
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

    .btn-secondary {
        background: hsl(var(--secondary));
        color: hsl(var(--secondary-foreground));
        border-color: hsl(var(--border));
    }

    .btn-secondary:hover {
        background: hsl(var(--muted));
    }

    .btn-icon {
        background: none;
        padding: 0.25rem;
        color: hsl(var(--muted-foreground));
    }

    .btn-icon:hover {
        color: hsl(var(--foreground));
    }

    .btn-icon.danger:hover {
        color: hsl(var(--destructive));
    }

    .toggle-group {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .toggle-label {
        font-size: 0.875rem;
        color: hsl(var(--foreground));
    }

    .toggle-desc {
        font-size: 0.8rem;
        color: hsl(var(--muted-foreground));
    }

    

    input[type="range"] {
        -webkit-appearance: none;
        appearance: none;
        width: 100%;
        height: 6px;
        background: hsl(var(--secondary));
        border-radius: 3px;
        outline: none;
        transition: opacity .2s;
        cursor: pointer;
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 18px;
        height: 18px;
        background: hsl(var(--primary));
        border-radius: 50%;
        border: 3px solid hsl(var(--background));
    }

    input[type="range"]::-moz-range-thumb {
        width: 18px;
        height: 18px;
        background: hsl(var(--primary));
        border-radius: 50%;
        border: 3px solid hsl(var(--background));
    }

    

    .icon {
        width: 1rem;
        height: 1rem;
        stroke: currentColor;
        stroke-width: 2;
        fill: none;
    }
 </style>