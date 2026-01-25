export interface GeneratorOptions {
  uppercase: boolean;
  lowercase: boolean;
  digits: boolean;
  symbols: boolean;
  ambiguous: boolean;
  similar: boolean;
  pronounceable: boolean;
}

export const SYMBOL_CHARSET = '!@#$%^&*()_+-=[]{}|;:,.<>?';
export const AMBIGUOUS_CHARS = new Set(['i', 'I', '1', 'L', 'o', 'O', '0']);
export const SIMILAR_CHARS = new Set('oO0l1IvVwWsScCpPkKxXzZbBdDgGqQeEfFtTuUjJmMnrRhHaAyY'.split(''));

export class GeneratorService {
  static generate(length: number, options: Partial<GeneratorOptions> = {}): string {
    const opts = {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      pronounceable: false,
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

  static calculateEntropy(length: number, poolSize: number): number {
    if (poolSize <= 0) return 0;
    return Math.floor(length * Math.log2(poolSize));
  }

  static getPoolSize(options: Partial<GeneratorOptions>): number {
    const opts = {
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: true,
      ambiguous: false,
      similar: false,
      ...options
    };
    
    let size = 0;
    if (opts.uppercase) size += 26;
    if (opts.lowercase) size += 26;
    if (opts.digits) size += 10;
    if (opts.symbols) size += SYMBOL_CHARSET.length;

    if (opts.ambiguous) {
      size = Math.max(0, size - 7);
    }
    return size;
  }
}
