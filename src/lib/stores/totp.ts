import { persistentWritable } from '$lib/utils/persistentStore';

export const loginTotpSecret = persistentWritable<string | null>('pulsar.loginTotpSecret', null);
export const loginTotpConfigured = persistentWritable<boolean>('pulsar.loginTotpConfigured', false);
