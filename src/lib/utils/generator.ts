export interface GeneratorOptions {
  uppercase: boolean;
  lowercase: boolean;
  digits: boolean;
  symbols: boolean;
  ambiguous: boolean;
  similar: boolean;
  pronounceable: boolean;
  mode: 'password' | 'passphrase';
  wordCount: number;
  separator: string;
}

export const SYMBOL_CHARSET = '!@#$%^&*()_+-=[]{}|;:,.<>?';
export const AMBIGUOUS_CHARS = new Set(['i', 'I', '1', 'L', 'o', 'O', '0']);
export const SIMILAR_CHARS = new Set(
  'oO0l1IvVwWsScCpPkKxXzZbBdDgGqQeEfFtTuUjJmMnrRhHaAyY'.split('')
);

export class GeneratorService {
  static generate(length: number, options: Partial<GeneratorOptions> = {}): string {
    const opts: GeneratorOptions = {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      pronounceable: false,
      mode: 'password',
      wordCount: 4,
      separator: '-',
      ...options
    };

    if (opts.mode === 'passphrase') {
      return '';
    }

    let charset = '';
    if (opts.uppercase) charset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    if (opts.lowercase) charset += 'abcdefghijklmnopqrstuvwxyz';
    if (opts.digits) charset += '0123456789';
    if (opts.symbols) charset += SYMBOL_CHARSET;

    if (opts.ambiguous) {
      charset = charset
        .split('')
        .filter((c) => !AMBIGUOUS_CHARS.has(c))
        .join('');
    }

    if (opts.similar) {
      charset = charset
        .split('')
        .filter((c) => !SIMILAR_CHARS.has(c))
        .join('');
    }

    if (charset.length === 0) return '';

    let password = '';
    const array = new Uint32Array(length);
    crypto.getRandomValues(array);

    if (opts.pronounceable) {
      const vowels = 'aeiou';
      const consonants = charset
        .split('')
        .filter((c) => !vowels.includes(c.toLowerCase()))
        .join('');
      const actualVowels = charset
        .split('')
        .filter((c) => vowels.includes(c.toLowerCase()))
        .join('');

      if (actualVowels.length > 0 && consonants.length > 0) {
        for (let i = 0; i < length; i++) {
          const source = i % 2 === 0 ? consonants : actualVowels;
          password += source[array[i] % source.length];
        }
      } else {
        for (let i = 0; i < length; i++) {
          password += charset[array[i] % charset.length];
        }
      }
    } else {
      for (let i = 0; i < length; i++) {
        password += charset[array[i] % charset.length];
      }
    }

    return password;
  }

  static async generatePassphrase(wordCount: number, separator: string): Promise<string> {
    const { EFF_LARGE_WORDLIST } = await import('./wordlist');
    const array = new Uint32Array(wordCount);
    crypto.getRandomValues(array);
    const words: string[] = [];
    for (let i = 0; i < wordCount; i++) {
      words.push(EFF_LARGE_WORDLIST[array[i] % EFF_LARGE_WORDLIST.length]);
    }
    return words.join(separator);
  }

  static async calculateEntropy(
    length: number,
    poolSize: number,
    mode: 'password' | 'passphrase' = 'password'
  ): Promise<number> {
    if (mode === 'passphrase') {
      const { EFF_LARGE_WORDLIST } = await import('./wordlist');
      return Math.floor(length * Math.log2(EFF_LARGE_WORDLIST.length));
    }
    if (poolSize <= 0) return 0;
    return Math.floor(length * Math.log2(poolSize));
  }

  static async getPoolSize(options: Partial<GeneratorOptions>): Promise<number> {
    if (options.mode === 'passphrase') {
      const { EFF_LARGE_WORDLIST } = await import('./wordlist');
      return EFF_LARGE_WORDLIST.length;
    }

    const opts = {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      ...options
    };

    let charset = '';
    if (opts.uppercase) charset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    if (opts.lowercase) charset += 'abcdefghijklmnopqrstuvwxyz';
    if (opts.digits) charset += '0123456789';
    if (opts.symbols) charset += SYMBOL_CHARSET;

    if (opts.ambiguous) {
      charset = charset
        .split('')
        .filter((c) => !AMBIGUOUS_CHARS.has(c))
        .join('');
    }

    if (opts.similar) {
      charset = charset
        .split('')
        .filter((c) => !SIMILAR_CHARS.has(c))
        .join('');
    }

    return charset.length;
  }
}
