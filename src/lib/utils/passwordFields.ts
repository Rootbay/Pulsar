import type { PasswordItem } from '$lib/types/password';
import type { DisplayField, TotpDisplayField } from '$lib/types/password-fields';

const DEFAULT_TOTP_DIGITS = 6;
const DEFAULT_TOTP_PERIOD = 30;

function createTotpField(secret: string, icon: string): TotpDisplayField {
  return {
    id: 'totp',
    name: 'Authenticator (TOTP)',
    value: secret,
    type: 'totp',
    icon,
    meta: {
      secret,
      digits: DEFAULT_TOTP_DIGITS,
      period: DEFAULT_TOTP_PERIOD
    }
  };
}

export function buildDisplayFields(
  item: PasswordItem | null,
  icons: Record<string, string>
): DisplayField[] {
  if (!item) {
    return [];
  }

  let staticFields: DisplayField[] = [];

  const category = item.category || 'login';

  if (category === 'login') {
    staticFields = [
      {
        id: 'username',
        name: 'Username',
        value: item.username ?? null,
        type: 'text',
        icon: icons.user
      },
      {
        id: 'password',
        name: 'Password',
        value: item.password ?? null,
        type: 'password',
        icon: icons.key
      },
      ...(item.totp_secret ? [createTotpField(item.totp_secret, icons.security)] : []),
      {
        id: 'url',
        name: 'URL',
        value: item.url ?? null,
        type: 'text',
        icon: icons.link
      }
    ];
  } else if (category === 'card') {
    staticFields = [
      {
        id: 'username',
        name: 'Cardholder Name',
        value: item.username ?? null,
        type: 'text',
        icon: icons.user
      },
      {
        id: 'password',
        name: 'Card Number',
        value: item.password ?? null,
        type: 'password',
        icon: icons.key
      }
    ];
  } else if (category === 'identity') {
    staticFields = [
      {
        id: 'password',
        name: 'Full Name',
        value: item.password ?? null,
        type: 'text',
        icon: icons.user
      },
      {
        id: 'username',
        name: 'Email',
        value: item.username ?? null,
        type: 'text',
        icon: icons.user
      }
    ];
  }

  staticFields.push({
    id: 'notes',
    name: category === 'note' ? 'Secure Note' : 'Notes',
    value: item.notes ?? null,
    type: 'multiline',
    icon: icons.notes
  });

  const customFields: DisplayField[] = (item.custom_fields ?? []).map((field) => ({
    id: field.name,
    name: field.name,
    value: field.value ?? null,
    type: field.field_type,
    icon: icons.plus
  }));

  let allFields = [...staticFields, ...customFields];

  if (item.field_order?.length) {
    const ordered: DisplayField[] = [];
    const map = new Map(allFields.map((field) => [field.id, field]));

    for (const fieldId of item.field_order) {
      const field = map.get(fieldId);
      if (field) {
        ordered.push(field);
        map.delete(fieldId);
      }
    }

    ordered.push(...map.values());
    allFields = ordered;
  }

  return allFields;
}
