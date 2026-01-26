export type FilterCategory = 'all' | 'recent';

class AppState {
    selectedTag = $state<string | null>(null);
    currentView = $state<string>('passwords');
    filterCategory = $state<FilterCategory>('all');
    isLocked = $state<boolean>(true);
    isDatabaseLoaded = $state<boolean>(false);
    needsPasswordSetup = $state<boolean>(false);
    showSettingsPopup = $state<boolean>(false);
    totpVerified = $state<boolean>(false);
    totpRequired = $state<boolean>(false);

    /**
     * Computed state to check if the application is fully unlocked and ready.
     */
    get isReady() {
        return this.isDatabaseLoaded && !this.isLocked && (!this.totpRequired || this.totpVerified);
    }

    /**
     * Resets the UI state to a clean slate, usually called when locking or switching vaults.
     */
    resetUI() {
        this.selectedTag = null;
        this.currentView = 'passwords';
        this.filterCategory = 'all';
    }
}

export const appState = new AppState();
