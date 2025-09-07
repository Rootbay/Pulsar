<script lang="ts">
	import { settingsStore } from '$lib/stores';
	import { appearanceSettings, type AppearanceSettingsStore } from '$lib/stores/appearance';
	import { onMount } from 'svelte';
	import Switch from '$lib/components/ui/Switch.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import SettingItem from '$lib/components/ui/SettingItem.svelte';
	import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
	import { iconPaths } from '$lib/icons';

	let theme: 'system' | 'light' | 'dark';
	let compactMode: boolean;
	let fontSize: number;
	let highContrast: boolean;
	let reducedMotion: boolean;
	let pageDensity: 'comfortable' | 'compact' | 'dense';

	appearanceSettings.subscribe((settings) => {
		theme = settings.theme;
		compactMode = settings.compactMode;
		fontSize = settings.fontSize;
		highContrast = settings.highContrast;
		reducedMotion = settings.reducedMotion;
		pageDensity = settings.pageDensity;
	});

	onMount(() => {
		settingsStore.registerModule('appearance', appearanceSettings);
	});

	function handleChange() {
		appearanceSettings.set({
			theme,
			compactMode,
			fontSize,
			highContrast,
			reducedMotion,
			pageDensity
		});
	}
</script>

<div class="appearance-container">
	<SettingsCard icon={iconPaths.appearance} title="Theme & Display" description="Customize how the app looks and behaves.">
		<div class="settings-group">
			<SettingItem>
				<div slot="info" class="setting-info">
					<div class="setting-icon">
						<svg
							width="24"
							height="24"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z" /></svg
						>
					</div>
					<div>
						<div class="setting-label">Theme</div>
						<div class="setting-description">Choose how the app appears</div>
					</div>
				</div>
				<div slot="control" class="setting-control">
					<Select bind:value={theme} options={[{value: 'system', label: 'System'}, {value: 'light', label: 'Light'}, {value: 'dark', label: 'Dark'}]} on:change={handleChange} ariaLabel="Select theme" />
				</div>
			</SettingItem>

			<SettingItem>
				<div slot="info" class="setting-info">
					<div class="setting-icon">
						<svg
							width="24"
							height="24"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3" /></svg
						>
					</div>
					<div>
						<div class="setting-label">Compact Mode</div>
						<div class="setting-description">Reduce spacing and padding</div>
					</div>
				</div>
				<div slot="control" class="setting-control">
					<Switch
						checked={compactMode}
						ariaLabel="Toggle compact mode"
						on:click={() => {
							compactMode = !compactMode;
							handleChange();
						}}
					/>
				</div>
			</SettingItem>

			<SettingItem>
				<div slot="info" class="setting-info">
					<div class="setting-icon">
						<svg
							width="24"
							height="24"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							><path d="M4 7V4h16v3M9 20h6M12 4v16" /></svg
						>
					</div>
					<div>
						<div class="setting-label">Font Size</div>
						<div class="setting-description">Adjust text size for readability</div>
					</div>
				</div>
				<div slot="control" class="setting-control">
					<div class="range-container">
						<input
							type="range"
							min="12"
							max="20"
							class="range"
							bind:value={fontSize}
							on:input={handleChange}
						/>
						<span class="range-value">{fontSize}px</span>
					</div>
				</div>
			</SettingItem>
		</div>
	</SettingsCard>

	<SettingsCard icon={iconPaths.user} title="Accessibility" description="Fine-tune the app for your specific needs.">
		<div class="settings-group">
			<SettingItem>
				<div slot="info" class="setting-info">
					<div class="setting-icon">
						<svg
							width="24"
							height="24"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							><circle cx="12" cy="12" r="10" /><path d="M12 18a6 6 0 0 0 0-12v12z" /></svg
						>
					</div>
					<div>
						<div class="setting-label">High Contrast</div>
						<div class="setting-description">Increase contrast for better visibility</div>
					</div>
				</div>
				<div slot="control" class="setting-control">
					<Switch
						checked={highContrast}
						ariaLabel="Toggle high contrast"
						on:click={() => {
							highContrast = !highContrast;
							handleChange();
						}}
					/>
				</div>
			</SettingItem>

			<SettingItem>
				<div slot="info" class="setting-info">
					<div class="setting-icon">
						<svg
							width="24"
							height="24"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							><path d="m13 17-5-5 5-5m-5 5h12.5" /></svg
						>
					</div>
					<div>
						<div class="setting-label">Reduced Motion</div>
						<div class="setting-description">Minimize animations and transitions</div>
					</div>
				</div>
				<div slot="control" class="setting-control">
					<Switch
						checked={reducedMotion}
						ariaLabel="Toggle reduced motion"
						on:click={() => {
							reducedMotion = !reducedMotion;
							handleChange();
						}}
					/>
				</div>
			</SettingItem>
		</div>
	</SettingsCard>

	<SettingsCard icon={iconPaths.reorder} title="Page Density" description="Choose how content is displayed on each page.">

		<div class="preview-grid">
			<button
				class="preview-card"
				role="radio"
				aria-checked={pageDensity === 'comfortable'}
				aria-label="Set page density to comfortable"
				class:selected={pageDensity === 'comfortable'}
				on:click={() => {
					pageDensity = 'comfortable';
					handleChange();
				}}
				data-density="comfortable"
			>
				<div class="preview-title">Comfortable</div>
				<div class="preview-viz">
					<div></div>
					<div></div>
					<div></div>
				</div>
			</button>

			<button
				class="preview-card"
				role="radio"
				aria-checked={pageDensity === 'compact'}
				aria-label="Set page density to compact"
				class:selected={pageDensity === 'compact'}
				on:click={() => {
					pageDensity = 'compact';
					handleChange();
				}}
				data-density="compact"
			>
				<div class="preview-title">Compact</div>
				<div class="preview-viz">
					<div></div>
					<div></div>
					<div></div>
				</div>
			</button>

			<button
				class="preview-card"
				role="radio"
				aria-checked={pageDensity === 'dense'}
				aria-label="Set page density to dense"
				class:selected={pageDensity === 'dense'}
				on:click={() => {
					pageDensity = 'dense';
					handleChange();
				}}
				data-density="dense"
			>
				<div class="preview-title">Dense</div>
				<div class="preview-viz">
					<div></div>
					<div></div>
					<div></div>
				</div>
			</button>
		</div>
	</SettingsCard>
</div>

<style>
    

    * {
        box-sizing: border-box;
    }

    .appearance-container {
        flex: 1;
        padding: 32px;
        overflow-y: auto;
        position: relative;
        animation: slideUp 0.5s ease-out;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
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

    .setting-label {
        font-weight: 500;
        color: hsl(var(--foreground));
    }

    .setting-description {
        color: hsl(var(--muted-foreground));
        font-size: 0.875rem;
    }

    .setting-control {
        flex-shrink: 0;
        margin-left: 1rem;
    }

    

    

    .range-container {
        display: flex;
        align-items: center;
        gap: 1rem;
        min-width: 200px;
    }

    .range {
        flex: 1;
        height: 6px;
        border-radius: 3px;
        background: hsl(var(--secondary));
        outline: none;
        -webkit-appearance: none;
        appearance: none;
    }

    .range::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 20px;
        height: 20px;
        border-radius: 50%;
        background: hsl(var(--primary));
        cursor: pointer;
        border: 2px solid hsl(var(--foreground));
    }

    .range-value {
        color: hsl(var(--muted-foreground));
        font-size: 0.875rem;
        min-width: 2.5rem;
        text-align: right;
    }

    

    .preview-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 1rem;
    }

    .preview-card {
        background: hsl(var(--muted) / 0.3);
        border: 2px solid hsl(var(--border));
        border-radius: var(--radius);
        padding: 1rem;
        text-align: center;
        cursor: pointer;
        transition: var(--transition-smooth);
    }

    .preview-card:hover {
        border-color: hsl(var(--primary) / 0.5);
        transform: translateY(-3px);
    }

    .preview-card.selected {
        border-color: hsl(var(--primary));
        background: hsl(var(--primary) / 0.1);
        box-shadow: var(--shadow-glow);
    }

    .preview-title {
        font-weight: 500;
        margin-bottom: 0.75rem;
    }

    .preview-viz {
        display: flex;
        flex-direction: column;
        gap: 4px;
        background: hsl(var(--background) / 0.5);
        padding: 0.5rem;
        border-radius: calc(var(--radius) - 4px);
    }

    .preview-viz div {
        background: hsl(var(--muted-foreground) / 0.5);
        border-radius: 3px;
        height: 6px;
    }

    .preview-card[data-density="comfortable"] .preview-viz {
        gap: 6px;
    }

    .preview-card[data-density="compact"] .preview-viz {
        gap: 4px;
    }

    .preview-card[data-density="dense"] .preview-viz {
        gap: 2px;
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

    @media (max-width: 640px) {
        
        .setting-control {
            margin-left: 0;
        }
    }
</style>