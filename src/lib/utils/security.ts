import { zxcvbn, type ZxcvbnResult } from '@zxcvbn-ts/core';
import { zxcvbnOptions } from '@zxcvbn-ts/core';
import * as zxcvbnCommonPackage from '@zxcvbn-ts/language-common';

// Initialize zxcvbn with common language package
const { adjacencyGraphs, dictionary } = zxcvbnCommonPackage;
zxcvbnOptions.setOptions({
    dictionary: {
        ...dictionary,
    },
    graphs: adjacencyGraphs,
});

export interface PasswordHealth {
    score: number; // 0-4
    crackTimeDisplay: string;
    suggestions: string[];
    warning: string;
    isBreached: boolean | null; // null = not checked, false = safe, true = pwned
    breachCount: number;
}

export class SecurityService {
    /**
     * Estimates password strength using zxcvbn
     */
    static checkStrength(password: string, userInputs: string[] = []): ZxcvbnResult {
        return zxcvbn(password, userInputs);
    }

    /**
     * Checks if a password has been exposed in a data breach via HIBP k-anonymity API.
     * Returns the number of times it was seen in breaches (0 if safe).
     * 
     * Privacy: Only sends the first 5 chars of the SHA-1 hash to the API.
     */
    static async checkBreach(password: string): Promise<number> {
        if (!password) return 0;

        try {
            const buffer = new TextEncoder().encode(password);
            const hashBuffer = await crypto.subtle.digest('SHA-1', buffer);
            const hashArray = Array.from(new Uint8Array(hashBuffer));
            const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('').toUpperCase();

            const prefix = hashHex.slice(0, 5);
            const suffix = hashHex.slice(5);

            const response = await fetch(`https://api.pwnedpasswords.com/range/${prefix}`);
            if (!response.ok) {
                console.error('HIBP API error:', response.statusText);
                return 0; // Fail safe
            }

            const text = await response.text();
            const lines = text.split('\n');
            
            for (const line of lines) {
                const [lineSuffix, count] = line.split(':');
                if (lineSuffix.trim() === suffix) {
                    return parseInt(count, 10);
                }
            }

            return 0;
        } catch (err) {
            console.error('Failed to check breach status:', err);
            return 0;
        }
    }
}
