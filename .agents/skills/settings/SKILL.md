---
name: settings
description: How to add new settings to Pulsar. Covers the AllSettings interface, default values, SettingsStore deep-merge, settings UI pages, and backend persistence.
---

# Adding New Settings

The settings system uses a deep-merge pattern that makes adding new settings safe and backward-compatible.

## Step 1: Define the Setting Interface

Edit `src/lib/config/settings.ts`:

```typescript
// Add to existing interface or create new one
export interface MyNewSettings {
  myNewOption: boolean;
  myNewValue: number;
}

// Add defaults
export const defaultMyNewSettings: MyNewSettings = {
  myNewOption: false,
  myNewValue: 42,
};

// Add to AllSettings interface
export interface AllSettings {
  // ... existing sections ...
  myNew: MyNewSettings;  // ← add here
}
```

## Step 2: Register Defaults in SettingsStore

Edit `src/lib/stores/appSettings.svelte.ts`:

```typescript
import { defaultMyNewSettings } from '../config/settings';

const defaultAllSettings: AllSettings = {
  // ... existing defaults ...
  myNew: defaultMyNewSettings,  // ← add here
};
```

**That's it for the data layer.** The `SettingsStore` uses `#deepMerge()` to merge stored settings with defaults, so:
- Existing users get the new defaults automatically
- New fields are added without breaking existing data
- Removed fields are silently dropped

## Step 3: Use the Setting

```typescript
import { settings } from '$lib/stores/appSettings.svelte';

// Read
const value = settings.state.myNew.myNewOption;

// Write (triggers debounced save)
settings.state.myNew.myNewOption = true;
settings.save();

// Or use the update helper
settings.update('myNew', { ...settings.state.myNew, myNewOption: true });
```

## Step 4: Create Settings UI Page

Settings pages live in `src/routes/settings/{section}/+page.svelte`:

```svelte
<script lang="ts">
  import { settings } from '$lib/stores/appSettings.svelte';
  import SettingsCard from '$lib/components/ui/SettingsCard.svelte';
  import SettingItem from '$lib/components/ui/SettingItem.svelte';
  import Switch from '$lib/components/ui/Switch.svelte';
  import { t } from '$lib/i18n.svelte';
  import { i18n } from '$lib/i18n.svelte';

  const locale = $derived(i18n.locale);
</script>

<SettingsCard title={t(locale, 'my_new_settings_title')}>
  <SettingItem label={t(locale, 'my_new_option_label')}>
    <Switch
      checked={settings.state.myNew.myNewOption}
      onCheckedChange={(checked) => {
        settings.state.myNew.myNewOption = checked;
        settings.save();
      }}
    />
  </SettingItem>
</SettingsCard>
```

## Step 5: Add to Settings Navigation

The settings layout in `src/routes/settings/+layout.svelte` contains the navigation sidebar. Add your new section to the navigation items list.

## Backend Settings (if needed)

If the setting affects the Rust backend (e.g., startup behavior, tray icon):

1. The frontend saves settings via `callBackend('set_all_settings', { settings: JSON.stringify(snapshot) })`
2. Then calls `callBackend('apply_system_settings')` to apply changes
3. The Rust side reads settings in `src-tauri/src/settings/mod.rs` and `system.rs`
4. Add handling logic to `apply_system_settings` command

## Existing Settings Sections

| Section | Interface | File |
|---------|-----------|------|
| Advanced | `AdvancedSettings` | KDF presets, memory security |
| Appearance | `AppearanceSettings` | Theme, density, font size, panel widths |
| Autofill | `AutofillSettings` | Browser autofill, global autotype |
| Backup | `BackupSettings` | Auto backups, WebDAV sync config |
| Clipboard | `ClipboardSettings` | Clear duration, history blocking |
| General | `GeneralSettings` | Language, startup vault, system tray |
| Generator | `GeneratorSettings` | Password length, character options |
| Security | `SecuritySettings` | Auto-lock, biometric, TOTP |
| Vault | `VaultSettings` | Per-vault: name, TOTP, backups |
