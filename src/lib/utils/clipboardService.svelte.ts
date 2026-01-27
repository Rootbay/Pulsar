import { callBackend } from './backend';
import type { ClipboardSettings } from '$lib/config/settings';
import { settings } from '$lib/stores/appSettings.svelte';

interface ClipboardPolicyStatus {
  integrationAvailable: boolean;
  historyBlockingSupported: boolean;
  historyBlockingActive: boolean;
}

export interface ClipboardIntegrationState extends ClipboardPolicyStatus {
  applying: boolean;
  lastError: string | null;
}

const defaultStatus: ClipboardIntegrationState = {
  integrationAvailable: true,
  historyBlockingSupported: false,
  historyBlockingActive: false,
  applying: false,
  lastError: null
};

class ClipboardService {
  state = $state<ClipboardIntegrationState>(defaultStatus);
  ready = $state(false);
  #initialized = false;

  private updateState(partial: Partial<ClipboardIntegrationState>) {
    this.state = { ...this.state, ...partial };
  }

  private updateFromStatus(status: ClipboardPolicyStatus) {
    this.updateState({
      integrationAvailable: status.integrationAvailable,
      historyBlockingSupported: status.historyBlockingSupported,
      historyBlockingActive: status.historyBlockingActive,
      lastError: null
    });
  }

  private async applyPolicy(clipboardSettings: ClipboardSettings): Promise<ClipboardPolicyStatus> {
    return callBackend<ClipboardPolicyStatus>('apply_clipboard_policy', {
      payload: {
        clipboardIntegration: clipboardSettings.clipboardIntegration,
        blockHistory: clipboardSettings.blockHistory,
        onlyUnlocked: clipboardSettings.onlyUnlocked
      }
    });
  }

  async init(): Promise<void> {
    if (this.#initialized) return;

    try {
      const status = await callBackend<ClipboardPolicyStatus>('get_clipboard_capabilities');
      this.updateFromStatus(status);
    } catch (error) {
      const message = this.extractErrorMessage(error);
      this.updateState({
        integrationAvailable: false,
        lastError: message
      });
    }

    const currentSettings = settings.state.clipboard;

    try {
      const status = await this.applyPolicy(currentSettings);
      this.updateFromStatus(status);
    } catch (error) {
      const message = this.extractErrorMessage(error);
      this.updateState({
        lastError: message
      });

      if (currentSettings.blockHistory && message.toLowerCase().includes('not supported')) {
        settings.state.clipboard.blockHistory = false;
        settings.save();

        const sanitized = settings.state.clipboard;
        try {
          const status = await this.applyPolicy(sanitized);
          this.updateFromStatus(status);
        } catch (secondaryError) {
          const secondaryMessage = this.extractErrorMessage(secondaryError);
          this.updateState({
            lastError: secondaryMessage
          });
        }
      }
    }

    this.#initialized = true;
    this.ready = true;
  }

  async updateSettings(partial: Partial<ClipboardSettings>): Promise<void> {
    const current = settings.state.clipboard;
    const next: ClipboardSettings = { ...current, ...partial };

    this.updateState({ applying: true, lastError: null });

    try {
      const status = await this.applyPolicy(next);
      settings.state.clipboard = next;
      settings.save();

      this.updateFromStatus(status);
    } catch (error) {
      const message = this.extractErrorMessage(error);
      this.updateState({ lastError: message });
      throw new Error(message);
    } finally {
      this.updateState({ applying: false });
    }
  }

  async clearNow(): Promise<void> {
    await callBackend('clear_clipboard');
  }

  private extractErrorMessage(error: unknown): string {
    if (error instanceof Error) return error.message;
    if (typeof error === 'string') return error;
    try {
      return JSON.stringify(error);
    } catch (jsonError) {
      console.error('Failed to stringify clipboard error payload', jsonError);
      return 'Unknown clipboard error';
    }
  }
}

export const clipboardService = new ClipboardService();
export const initClipboardService = () => clipboardService.init();
export const updateClipboardSettings = (p: Partial<ClipboardSettings>) =>
  clipboardService.updateSettings(p);
export const clearClipboardNow = () => clipboardService.clearNow();
export const clipboardIntegrationState = {
  get value() {
    return clipboardService.state;
  }
};
export const clipboardServiceReady = {
  get value() {
    return clipboardService.ready;
  }
};
