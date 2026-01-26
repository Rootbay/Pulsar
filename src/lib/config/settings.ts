export interface AdvancedSettings {
  kdfPreset: 'fast' | 'balanced' | 'secure' | 'paranoid';
  timeCost: number;
  memoryCost: number;
  parallelism: number;
  wipeConfirmationText: string;
  lockMemoryPages: boolean;
  secureMemoryAllocation: boolean;
}

export const defaultAdvancedSettings: AdvancedSettings = {
  kdfPreset: 'balanced',
  timeCost: 3,
  memoryCost: 64,
  parallelism: 4,
  wipeConfirmationText: '',
  lockMemoryPages: true,
  secureMemoryAllocation: true
};

export interface AppearanceSettings {
  theme: 'system' | 'light' | 'dark';
  compactMode: boolean;
  fontSize: number;
  highContrast: boolean;
  reducedMotion: boolean;
  pageDensity: 'comfortable' | 'compact' | 'dense';
}

export const defaultAppearanceSettings: AppearanceSettings = {
  theme: 'dark',
  compactMode: false,
  fontSize: 14,
  highContrast: false,
  reducedMotion: false,
  pageDensity: 'comfortable'
};

export interface AutofillSettings {
  browserAutofill: boolean;
  globalAutotype: boolean;
  osUnlock: boolean;
  perSiteConfirmation: boolean;
}

export const defaultAutofillSettings: AutofillSettings = {
  browserAutofill: false,
  globalAutotype: false,
  osUnlock: true,
  perSiteConfirmation: true
};

export interface BackupSettings {
  automaticBackups: boolean;
  backupFrequency: string;
  retentionCount: number;
  enablePlaintextExport: boolean;
  syncMode: string;
  selectedProvider: string | null;
}

export const defaultBackupSettings: BackupSettings = {
  automaticBackups: true,
  backupFrequency: 'daily',
  retentionCount: 7,
  enablePlaintextExport: false,
  syncMode: 'off',
  selectedProvider: null
};

export interface ClipboardSettings {
  clipboardIntegration: boolean;
  clearAfterDuration: number;
  blockHistory: boolean;
  onlyUnlocked: boolean;
  permissionLevel: string;
}

export const defaultClipboardSettings: ClipboardSettings = {
  clipboardIntegration: true,
  clearAfterDuration: 12,
  blockHistory: true,
  onlyUnlocked: true,
  permissionLevel: 'ask'
};

export type AppLanguage =
  | 'system'
  | 'en'
  | 'sv'
  | 'es'
  | 'fr'
  | 'de'
  | 'pt-BR'
  | 'zh'
  | 'ru'
  | 'ja'
  | 'hi'
  | 'ko'
  | 'ar'
  | 'it'
  | 'tr'
  | 'nl'
  | 'pl'
  | 'id'
  | 'th'
  | 'vi'
  | 'el';

export interface GeneralSettings {
  appLanguage: AppLanguage;
  defaultVaultOnStartup: string;
  startOnSystemBoot: boolean;
  showInSystemTray: boolean;
  defaultViewOnOpen: string;
  enable2FAForUnlock: boolean;
  totpEnabled: boolean;
}

export const defaultGeneralSettings: GeneralSettings = {
  appLanguage: 'system',
  defaultVaultOnStartup: '16 characters',
  startOnSystemBoot: true,
  showInSystemTray: true,
  defaultViewOnOpen: '16 characters',
  enable2FAForUnlock: false,
  totpEnabled: true
};

export interface GeneratorSettings {
  passwordLength: number;
  options: {
    uppercase: boolean;
    lowercase: boolean;
    digits: boolean;
    symbols: boolean;
    ambiguous: boolean;
    similar: boolean;
    pronounceable: boolean;
  };
}

export const defaultGeneratorSettings: GeneratorSettings = {
  passwordLength: 47,
  options: {
    uppercase: false,
    lowercase: true,
    digits: false,
    symbols: false,
    ambiguous: false,
    similar: false,
    pronounceable: false
  }
};

export interface SecuritySettings {
  lockOnSuspend: boolean;
  lockOnMinimize: boolean;
  lockGraceSeconds: number;
  autoLockInactivity: string;
  biometricUnlock: boolean;
  sessionPersistence: boolean;
  externalBreachCheck: boolean;
  localReuseDetection: boolean;
  secureRAMHandling: boolean;
  clearClipboardOnCopy: boolean;
  clipboardClearTime: number;
  useTotp: boolean;
}

export const defaultSecuritySettings: SecuritySettings = {
  lockOnSuspend: true,
  lockOnMinimize: false,
  lockGraceSeconds: 5,
  autoLockInactivity: '5 minutes',
  biometricUnlock: true,
  sessionPersistence: false,
  externalBreachCheck: false,
  localReuseDetection: true,
  secureRAMHandling: true,
  clearClipboardOnCopy: false,
  clipboardClearTime: 0,
  useTotp: false
};

export interface VaultSettings {
  name: string;
  totp: boolean;
  backups: boolean;
  compression: boolean;
}

export const defaultVaultSettings: VaultSettings = {
  name: 'New Vault',
  totp: true,
  backups: false,
  compression: false
};

export type VaultSettingsMap = Record<string, VaultSettings>;

export const defaultVaultSettingsMap: VaultSettingsMap = {};

export interface PasswordPreset {
  name: string;
  length: number;
  charSet: string;
  strength: number;
  settings: {
    uppercase: boolean;
    lowercase: boolean;
    digits: boolean;
    symbols: boolean;
    ambiguous: boolean;
    similar: boolean;
    pronounceable: boolean;
  };
}

export const defaultPasswordPresets: PasswordPreset[] = [
  {
    name: 'Recommended 20',
    length: 20,
    charSet: 'All',
    strength: 98,
    settings: {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      pronounceable: false
    }
  },
  {
    name: 'Banking Safe',
    length: 12,
    charSet: 'Alphanumeric',
    strength: 85,
    settings: {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: false,
      ambiguous: false,
      similar: false,
      pronounceable: false
    }
  },
  {
    name: 'Ultra Secure',
    length: 32,
    charSet: 'All + Extended',
    strength: 99,
    settings: {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      pronounceable: false
    }
  }
];

export interface SiteRule {
  url: string;
  length: number;
  type: string;
  desc: string;
  settings: {
    uppercase: boolean;
    lowercase: boolean;
    digits: boolean;
    symbols: boolean;
    ambiguous: boolean;
    similar: boolean;
    pronounceable: boolean;
  };
}

export const defaultSiteRules: SiteRule[] = [
  {
    url: 'banking.example.com',
    length: 12,
    type: 'Letters + Numbers',
    desc: 'Banking sites typically require letters and numbers only',
    settings: {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: false,
      ambiguous: false,
      similar: false,
      pronounceable: false
    }
  },
  {
    url: 'social.example.com',
    length: 20,
    type: 'All characters',
    desc: 'Social media sites allow complex passwords',
    settings: {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      pronounceable: false
    }
  },
  {
    url: 'work.example.com',
    length: 14,
    type: 'Letters + Numbers + Basic symbols',
    desc: 'Corporate policies often limit symbol types',
    settings: {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      pronounceable: false
    }
  }
];
