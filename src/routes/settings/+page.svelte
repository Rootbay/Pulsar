<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import General from './general/+page.svelte';
	import Security from './security/+page.svelte';
	import VaultStorage from './vault/+page.svelte';
	import AutoFill from './autofill/+page.svelte';
	import Generator from './generator/+page.svelte';
	import Clipboard from './clipboard/+page.svelte';
	import Backup from './backup/+page.svelte';
	import Appearance from './appearance/+page.svelte';
	import Advanced from './advanced/+page.svelte';
	import About from './about/+page.svelte';
	import UnsavedChangesPopup from '$lib/components/UnsavedChangesPopup.svelte';
	import { settingsStore } from '$lib/stores';

	const dispatch = createEventDispatcher();
	let selectedSection = 'account';

	function showSection(sectionId: string) {
		selectedSection = sectionId;
	}

	function closeSettings() {
		dispatch('close');
	}

	function handleReset() {
		settingsStore.resetAll();
	}

	function handleSave() {
		settingsStore.saveAll();
	}
</script>

<div class="overlay" role="button" tabindex="0" aria-label="Close settings" on:click={(e) => { if (e.target === e.currentTarget) closeSettings(); }} on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') closeSettings(); }}>
    <div class="settings-container" role="document" tabindex="-1">
        <div class="main-content">
            <div class="sidebar">
                <button class="sidebar-item" class:active={selectedSection === 'account'} on:click={() => showSection('account')}>
                    <div class="sidebar-icon account">
                        <svg class="icon" viewBox="0 0 24 24">
                            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                            <circle cx="12" cy="7" r="4"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Account</div>
                        <div class="sidebar-subtitle">Profile & authentication</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'security'} on:click={() => showSection('security')}>
                    <div class="sidebar-icon security">
                        <svg class="icon" viewBox="0 0 24 24">
                            <path d="M9 12l2 2 4-4"/>
                            <path d="M21 12c-1 0-3-1-3-3s2-3 3-3 3 1 3 3-2 3-3 3"/>
                            <path d="M3 12c1 0 3-1 3-3s-2-3-3-3-3 1-3 3 2 3 3 3"/>
                            <path d="M12 3c0 1-1 3-3 3s-3-2-3-3 1-3 3-3 3 2 3 3"/>
                            <path d="M12 21c0-1 1-3 3-3s3 2 3 3-1 3-3 3-3-2-3-3"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Security</div>
                        <div class="sidebar-subtitle">Passwords & protection</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'vaultstorage'} on:click={() => showSection('vaultstorage')}>
                    <div class="sidebar-icon vaultstorage">
                        <svg class="icon" viewBox="0 0 24 24">
                            <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
                            <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Vault</div>
                        <div class="sidebar-subtitle">Secure storage</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'autofill'} on:click={() => showSection('autofill')}>
                    <div class="sidebar-icon autofill">
                        <svg class="icon" viewBox="0 0 24 24">
                            <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
                            <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Autofill</div>
                        <div class="sidebar-subtitle">Save & fill passwords</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'generator'} on:click={() => showSection('generator')}>
                    <div class="sidebar-icon generator">
                        <svg class="icon" viewBox="0 0 24 24">
                            <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
                            <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Generator</div>
                        <div class="sidebar-subtitle">Create strong passwords</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'clipboard'} on:click={() => showSection('clipboard')}>
                    <div class="sidebar-icon clipboard">
                        <svg class="icon" viewBox="0 0 24 24">
                            <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
                            <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Clipboard</div>
                        <div class="sidebar-subtitle">Manage clipboard items</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'backup'} on:click={() => showSection('backup')}>
                    <div class="sidebar-icon backup">
                        <svg class="icon" viewBox="0 0 24 24">
                            <path d="M17 17h5v-5"/>
                            <path d="m21 12-8 8-4-4"/>
                            <path d="M7 7h5v5"/>
                            <path d="m3 12 4-4 8 8"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Backup</div>
                        <div class="sidebar-subtitle">Backup data</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'appearance'} on:click={() => showSection('appearance')}>
                    <div class="sidebar-icon appearance">
                        <svg class="icon" viewBox="0 0 24 24">
                            <circle cx="13.5" cy="6.5" r=".5"/>
                            <circle cx="17.5" cy="10.5" r=".5"/>
                            <circle cx="8.5" cy="7.5" r=".5"/>
                            <circle cx="6.5" cy="12.5" r=".5"/>
                            <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Appearance</div>
                        <div class="sidebar-subtitle">Theme & display</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'advanced'} on:click={() => showSection('advanced')}>
                    <div class="sidebar-icon advanced">
                        <svg class="icon" viewBox="0 0 24 24">
                            <circle cx="12" cy="12" r="10"/>
                            <path d="m9 12 2 2 4-4"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">Advanced</div>
                        <div class="sidebar-subtitle">Advanced features</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>

                <button class="sidebar-item" class:active={selectedSection === 'about'} on:click={() => showSection('about')}>
                    <div class="sidebar-icon about">
                        <svg class="icon" viewBox="0 0 24 24">
                            <circle cx="12" cy="12" r="10"/>
                            <path d="m9 12 2 2 4-4"/>
                        </svg>
                    </div>
                    <div class="sidebar-text">
                        <div class="sidebar-title">About</div>
                        <div class="sidebar-subtitle">App info & support</div>
                    </div>
                    <svg class="sidebar-arrow" viewBox="0 0 24 24">
                        <path d="m9 18 6-6-6-6"/>
                    </svg>
                </button>
            </div>

            <div class="content-area">
                {#if selectedSection === 'account'}
                    <General />
                {:else if selectedSection === 'security'}
                    <Security />
                {:else if selectedSection === 'vaultstorage'}
                    <VaultStorage />
                {:else if selectedSection === 'autofill'}
                    <AutoFill />
                {:else if selectedSection === 'generator'}
                    <Generator />
                {:else if selectedSection === 'clipboard'}
                    <Clipboard />
                {:else if selectedSection === 'appearance'}
                    <Appearance />
                {:else if selectedSection === 'backup'}
                    <Backup />
                {:else if selectedSection === 'advanced'}
                    <Advanced />
                {:else if selectedSection === 'about'}
                    <About />
                {/if}
            </div>
        </div>
		{#if $settingsStore}
			<UnsavedChangesPopup on:reset={handleReset} on:save={handleSave} />
		{/if}
    </div>
</div>

<style>
    :root {
        --background: 0 0% 5.9%;
        --foreground: 0 0% 98%;
        --card: 240 10% 3.9%;
        --card-foreground: 0 0% 98%;
        --popover: 240 10% 3.9%;
        --popover-foreground: 0 0% 98%;
        --primary: 262.1 83.3% 57.8%;
        --primary-foreground: 210 20% 98%;
        --secondary: 240 3.7% 15.9%;
        --secondary-foreground: 0 0% 98%;
        --muted: 240 3.7% 15.9%;
        --muted-foreground: 240 5% 64.9%;
        --accent: 240 3.7% 15.9%;
        --accent-foreground: 0 0% 98%;
        --destructive: 0 84.2% 60.2%;
        --destructive-foreground: 0 0% 98%;
        --warning: 38 92% 50%;
        --border: 240 3.7% 15.9%;
        --input: 240 3.7% 15.9%;
        --ring: 262.1 83.3% 57.8%;
        --radius: 0.5rem;
        --success: 142 76% 36%;
    }
    
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }

    .overlay {
        position: fixed;
        inset: 0;
        background: hsl(var(--background) / 0.8);
        backdrop-filter: blur(12px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 50;
        animation: fadeIn 0.5s ease-out;
    }

    .settings-container {
        position: relative;
        width: 90%;
        max-width: 1200px;
        height: 75vh;
        background: hsl(var(--card) / 0.95);
        backdrop-filter: blur(4px);
        border-radius: 20px;
        border: 1px solid hsl(var(--border) / 0.5);
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
        overflow: hidden;
        animation: scaleIn 0.5s ease-out;
    }

    .main-content {
        display: flex;
        height: 100%;
    }

    .sidebar {
        width: 320px;
        padding: 24px;
        background: linear-gradient(135deg, hsl(var(--card) / 0.3), hsl(var(--muted) / 0.1));
        border-right: 1px solid hsl(var(--border) / 0.5);
        overflow-y: auto;
    }

    .sidebar-item {
        width: 100%;
        padding: 16px;
        margin-bottom: 8px;
        border-radius: 16px;
        border: 2px solid transparent;
        background: transparent;
        cursor: pointer;
        transition: all 0.3s;
        display: flex;
        align-items: center;
        gap: 16px;
        text-align: left;
    }

    .sidebar-item:hover {
        background: hsl(var(--muted) / 0.3);
        transform: scale(1.02);
    }

    .sidebar-item.active {
        background: hsl(var(--primary) / 0.2);
        border-color: hsl(var(--primary) / 0.3);
        box-shadow: 0 8px 25px -8px hsl(var(--primary) / 0.3);
        transform: scale(1.02);
    }

    .sidebar-icon {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.3s;
    }

    .sidebar-icon.account {
        background: hsl(210 100% 50% / 0.1);
        color: hsl(210 100% 50%);
    }

    .sidebar-icon.security {
        background: hsl(120 100% 40% / 0.1);
        color: hsl(120 100% 40%);
    }

    .sidebar-icon.vaultstorage {
        background: hsl(30 100% 50% / 0.1);
        color: hsl(30 100% 50%);
    }

    .sidebar-icon.autofill {
        background: hsla(197, 98%, 25%, 0.1);
        color: hsl(197, 98%, 25%);
    }

    .sidebar-icon.generator {
        background: hsla(165, 93%, 33%, 0.1);
        color: hsl(165, 93%, 33%);
    }

    .sidebar-icon.clipboard {
        background: hsla(128, 97%, 44%, 0.1);
        color: hsl(128, 97%, 44%);
    }

    .sidebar-icon.backup {
        background: hsla(320, 96%, 44%, 0.1);
        color: hsl(320, 96%, 44%);
    }
    
    .sidebar-icon.appearance {
        background: hsl(280 100% 60% / 0.1);
        color: hsl(280 100% 60%);
    }

    .sidebar-icon.advanced {
        background: hsla(347, 99%, 39%, 0.1);
        color: hsl(347, 99%, 39%);
    }

    .sidebar-icon.about {
        background: hsl(0 0% 50% / 0.1);
        color: hsl(0 0% 50%);
    }

    .sidebar-text {
        flex: 1;
    }
    .sidebar-title {
        font-weight: 600;
        color: hsl(var(--foreground));
        margin-bottom: 2px;
        transition: color 0.3s;
    }
    .sidebar-item.active .sidebar-title {
        color: hsl(var(--primary));
    }
    .sidebar-subtitle {
        font-size: 12px;
        color: hsl(var(--muted-foreground));
    }
    .sidebar-arrow {
        width: 20px;
        height: 20px;
					fill: var(--white);
        color: hsl(var(--muted-foreground));
        transition: all 0.3s;
    }
    .sidebar-item.active .sidebar-arrow {
        color: hsl(var(--primary));
        transform: rotate(90deg);
					fill: hsl(var(--primary));
    }
    .content-area {
        flex: 1;
        overflow-y: auto;
        padding: 24px;
    }
    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }
    @keyframes scaleIn {
        from { transform: scale(0.95); opacity: 0; }
        to { transform: scale(1); opacity: 1; }
    }
    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
    }
    .icon {
        width: 24px;
        height: 24px;
        fill: none;
        stroke: currentColor;
        stroke-width: 2;
        stroke-linecap: round;
        stroke-linejoin: round;
    }
</style>