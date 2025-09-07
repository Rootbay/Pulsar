import { derived, readable } from 'svelte/store';
import { appSettings } from './appSettings';
import { defaultAppearanceSettings, type AppearanceSettings } from '../config/settings';

function createAppearanceSettingsStore() {
    const { subscribe } = derived(appSettings, ($appSettings) => $appSettings.appearance);

    const hasUnsavedChanges = readable(false, (setReadable) => {
        let initialSettings: AppearanceSettings;
        const unsubscribe = appSettings.subscribe(($appSettings) => {
            const currentSettings = $appSettings.appearance;
            if (!initialSettings) {
                initialSettings = currentSettings; // Capture initial state
            }
            setReadable(JSON.stringify(currentSettings) !== JSON.stringify(initialSettings));
        });
        return unsubscribe;
    });

    return {
        subscribe,
        set: (value: AppearanceSettings) => {
            appSettings.update((settings) => {
                settings.appearance = value;
                return settings;
            });
        },
        update: (callback: (settings: AppearanceSettings) => AppearanceSettings) => {
            appSettings.update((settings) => {
                settings.appearance = callback(settings.appearance);
                return settings;
            });
        },
        save: () => {
            // Since appSettings automatically saves on update, we just need to ensure
            // appSettings has the latest appearance settings. The `set` and `update`
            // methods already do this.
            // If there were external changes to appearanceSettings not via set/update,
            // we might need a dummy update to appSettings to trigger its save mechanism.
            // For now, assuming all changes go through set/update.
            appSettings.update(s => s); // Dummy update to trigger appSettings save
        },
        reset: () => {
            appSettings.update((settings) => {
                settings.appearance = defaultAppearanceSettings;
                return settings;
            });
        },
        hasUnsavedChanges,
    };
}

export type AppearanceSettingsStore = ReturnType<typeof createAppearanceSettingsStore>;

export const appearanceSettings = createAppearanceSettingsStore();