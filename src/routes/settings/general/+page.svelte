<script lang="ts">
  import { generalSettings } from '$lib/stores/general';
  import type { GeneralSettings } from '$lib/config/settings';
  import KeyboardShortcutsPopup from '$lib/components/KeyboardShortcutsPopup.svelte';
  import Switch from '$lib/components/ui/Switch.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
  import { iconPaths } from '$lib/icons';

  let currentGeneralSettings: GeneralSettings;
  generalSettings.subscribe(value => {
    currentGeneralSettings = value;
  });

  let showKeyboardShortcutsPopup = false;

  function toggleSwitch(setting: keyof GeneralSettings) {
    if (typeof currentGeneralSettings[setting] === 'boolean') {
      const newValue = !currentGeneralSettings[setting];
      generalSettings.set({ ...currentGeneralSettings, [setting]: newValue });
    }
  }

  function handleSelectChange(event: Event, setting: keyof GeneralSettings) {
    const target = event.target as HTMLSelectElement;
    generalSettings.set({ ...currentGeneralSettings, [setting]: target.value });
  }
</script>

<div class="Content">
  <section>
    <SettingsCard icon={iconPaths.settings} title="General Settings" description="">

      <div class="Section2">
        <div class="two">
          <h2>App Language</h2>
          <div style="margin-top: 8px; width: 100%;">
            <Select bind:value={currentGeneralSettings.appLanguage} options={[{value: '8 characters', label: '8 characters'}, {value: '12 characters', label: '12 characters'}, {value: '16 characters', label: '16 characters'}, {value: '20 characters', label: '20 characters'}, {value: '32 characters', label: '32 characters'}]} on:change={(e: Event) => handleSelectChange(e, 'appLanguage')} ariaLabel="Select app language" />
          </div>
        </div>

        <div class="two">
          <h2>Default Vault on Startup</h2>
          <div style="margin-top: 8px; width: 100%;">
            <Select bind:value={currentGeneralSettings.defaultVaultOnStartup} options={[{value: '8 characters', label: '8 characters'}, {value: '12 characters', label: '12 characters'}, {value: '16 characters', label: '16 characters'}, {value: '20 characters', label: '20 characters'}, {value: '32 characters', label: '32 characters'}]} on:change={(e: Event) => handleSelectChange(e, 'defaultVaultOnStartup')} ariaLabel="Select default vault on startup" />
          </div>
        </div>
      </div>

      <div class="Section3">
        <div>
          <h2>Start on System Boot</h2>
          <p>Launch automatically when your computer starts</p>
        </div>
        <Switch checked={currentGeneralSettings.startOnSystemBoot} ariaLabel="Toggle start on system boot" on:click={() => toggleSwitch('startOnSystemBoot')} />
      </div>

      <div class="Section3">
        <div>
          <h2>Show in System Tray</h2>
          <p>Keep app accessible from system tray</p>
        </div>
        <Switch checked={currentGeneralSettings.showInSystemTray} ariaLabel="Toggle show in system tray" on:click={() => toggleSwitch('showInSystemTray')} />
      </div>

      <div class="Section5">
        <h2>Default View on Open</h2>
        <div style="margin-top: 8px; width: 320px;">
          <Select bind:value={currentGeneralSettings.defaultViewOnOpen} options={[{value: '8 characters', label: '8 characters'}, {value: '12 characters', label: '12 characters'}, {value: '16 characters', label: '16 characters'}, {value: '20 characters', label: '20 characters'}, {value: '32 characters', label: '32 characters'}]} on:change={(e: Event) => handleSelectChange(e, 'defaultViewOnOpen')} ariaLabel="Select default view on open" />
        </div>
      </div>
    </SettingsCard>
  </section>

  <section>
    <SettingsCard icon={iconPaths.lock} title="Two-Factor Authentication" description="">

      <div class="Section3">
        <div>
          <h2>Enable 2FA for Unlock</h2>
          <p>Add an extra layer of security to vault access</p>
        </div>
        <Switch checked={currentGeneralSettings.enable2FAForUnlock} ariaLabel="Toggle enable 2FA for unlock" on:click={() => toggleSwitch('enable2FAForUnlock')} />
      </div>

      <div class="SectionH" data-display="flex">
        <h3>Authentication Methods</h3>

        <div class="y">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#7C3AED" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-smartphone h-5 w-5 text-primary" data-lov-id="src/pages/AppBehavior.tsx:163:26" data-lov-name="Smartphone" data-component-path="src/pages/AppBehavior.tsx" data-component-line="163" data-component-file="AppBehavior.tsx" data-component-name="Smartphone" data-component-content="%7B%22className%22%3A%22h-5%20w-5%20text-primary%22%7D"><rect width="14" height="20" x="5" y="2" rx="2" ry="2"></rect><path d="M12 18h.01"></path></svg>
          <div>
            <h2>TOTP (Time-based)</h2>
            <p>Built-in authenticator support</p>
          </div>
          <div class="z">
            <Switch checked={currentGeneralSettings.totpEnabled} ariaLabel="Toggle TOTP (Time-based)" on:click={() => toggleSwitch('totpEnabled')} />
          </div>
        </div>

        <h3>Authentication Methods</h3>

        <div class="y">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#7C3AED" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-smartphone h-5 w-5 text-primary" data-lov-id="src/pages/AppBehavior.tsx:163:26" data-lov-name="Smartphone" data-component-path="src/pages/AppBehavior.tsx" data-component-line="163" data-component-file="AppBehavior.tsx" data-component-name="Smartphone" data-component-content="%7B%22className%22%3A%22h-5%20w-5%20text-primary%22%7D"><rect width="14" height="20" x="5" y="2" rx="2" ry="2"></rect><path d="M12 18h.01"></path></svg>
          <div>
            <h2>TOTP (Time-based)</h2>
            <p>Built-in authenticator support</p>
          </div>
          <div class="z">
            <Switch checked={currentGeneralSettings.totpEnabled} ariaLabel="Toggle TOTP (Time-based)" on:click={() => toggleSwitch('totpEnabled')} />
          </div>
        </div>
      </div>

      <div class="Override" data-display="flex">
        <h2>Per-vault 2FA override: </h2>
        <h2 id="s2">Configure different 2FA requirements for individual vaults in the Vaults settings</h2>
                      <button on:click={() => alert('Navigate to Vaults settings')}>Vaults settings</button>
      </div>
    </SettingsCard>
  </section>

  <section>
		<SettingsCard icon={iconPaths.key} title="Keyboard Shortcuts" description="">
			<div class="ts">
				<p>Customize keyboard shortcuts for quick access to common actions.</p>
				<button on:click={() => showKeyboardShortcutsPopup = true}>Configure Shortcuts</button>
			</div>
		</SettingsCard>
  </section>
</div>

{#if showKeyboardShortcutsPopup}
  <KeyboardShortcutsPopup on:close={() => showKeyboardShortcutsPopup = false} />
{/if}

<style>
  * {
    padding: 0;
    margin: 0;
    box-sizing: border-box;
  }

  h2 {
    font-size: 13px;
  }

  

  .Section2 {
    display: flex;
    padding: 0px 24px;
    margin: 0 0 32px 0;
  }

  .two {
    width:50%;
    padding: 0 12px 0 0;
  }

  .Section3 {
    padding: 0px 24px 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .Section5 {
    padding: 0px 24px;
    margin: 0 0 24px 0;
  }

  .SectionH {
    display: none;
    flex-direction: column;
    padding: 0 15px;
    margin: 0 24px;
    margin-bottom: 16px;
    border-left: 2px solid #201338;
  }

  .SectionH h3 {
    font-size: 15px;
    font-weight: 500;
    margin-bottom: 13px;
  }

  .SectionH .y {
    padding: 15px 15px;
    display: flex;
    border: 1px solid hsl(var(--border) / 0.5);
    border-radius: 10px;
    margin: 0 0 12px 0;
  }

  .SectionH .y svg {
    margin: 0 15px 0 0;
    align-self: center;
  }

  .SectionH .y .z {
    display: flex;
    flex: 1;
    min-width: 0;
    flex-direction: row-reverse;
  }

  

  .Override {
    padding: 13px 0;
    background-color: #141417;
    display: none;
    gap: 5px;
    margin: 0 41px;
    justify-content: center;
    border-radius: 12px;
    margin-bottom: 24px;
  }

  .Override h2 {
    margin: 0;
    padding: 0;
    font-size: 13px;
    font-weight: normal;
    white-space: nowrap;
  }

  #s2 {
    font-weight:300;
  }

  .ts {
    margin: 0 24px;
    padding-bottom: 24px;
  }

  .ts p {
    margin-bottom: 20px;
  }

  .ts button {
    padding: 10px;
    background-color: #7C3AED;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: background 0.2s, transform 0.1s;
  }

  .ts button:hover {
    background-color: #9D4EDD;
    transform: translateY(-1px);
  }
  .ts button:active {
    background-color: #5B21B6;
    transform: translateY(0);
  }
</style>