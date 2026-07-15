---
name: i18n
description: How to add translations and new i18n keys to Pulsar's internationalization system. Covers the 20 supported languages, the t() function, and locale detection.
---

# Internationalization (i18n)

Pulsar supports 20 languages with English as the fallback. All user-facing text MUST be translated.

## Supported Languages

| Code | Language | File |
|------|----------|------|
| `en` | English | `src/lib/i18n/en.json` (primary) |
| `sv` | Swedish | `sv.json` |
| `es` | Spanish | `es.json` |
| `fr` | French | `fr.json` |
| `de` | German | `de.json` |
| `pt-BR` | Portuguese (Brazil) | `pt-BR.json` |
| `zh` | Chinese | `zh.json` |
| `ru` | Russian | `ru.json` |
| `ja` | Japanese | `ja.json` |
| `hi` | Hindi | `hi.json` |
| `ko` | Korean | `ko.json` |
| `ar` | Arabic | `ar.json` |
| `it` | Italian | `it.json` |
| `tr` | Turkish | `tr.json` |
| `nl` | Dutch | `nl.json` |
| `pl` | Polish | `pl.json` |
| `id` | Indonesian | `id.json` |
| `th` | Thai | `th.json` |
| `vi` | Vietnamese | `vi.json` |
| `el` | Greek | `el.json` |

## Adding a New Translation Key

### Step 1: Add to English (primary)

Add the key to `src/lib/i18n/en.json`:

```json
{
  "existing_key": "Existing text",
  "my_new_key": "My new text with {variable} support"
}
```

### Step 2: Add to All Other Languages

Add the same key with translated text to all 19 other JSON files. If translation is unavailable, the English fallback is used automatically.

### Step 3: Use in Components

```svelte
<script lang="ts">
  import { t } from '$lib/i18n.svelte';
  import { i18n } from '$lib/i18n.svelte';

  const locale = $derived(i18n.locale);
</script>

<!-- Simple text -->
<p>{t(locale, 'my_new_key')}</p>

<!-- With variables -->
<p>{t(locale, 'my_new_key', { variable: 'world' })}</p>
```

### Step 4: Use in TypeScript

```typescript
import { t, i18n } from '$lib/i18n.svelte';

const message = t(i18n.locale, 'my_new_key', { variable: 'world' });
```

## Key Naming Convention

- Use `snake_case` for all keys
- Group by feature: `settings_general_title`, `auth_login_button`
- Use descriptive names: `password_generator_length_label` not `pg_len`

## Variable Interpolation

Use `{variableName}` in translation strings:

```json
{
  "items_count": "You have {count} items",
  "welcome_user": "Welcome, {name}!"
}
```

## Locale Detection

The `i18n.locale` getter:
1. Checks `settings.state.general.appLanguage`
2. If set to `'system'`, detects from `navigator.languages` → `navigator.language` → `Intl.DateTimeFormat`
3. Falls back to `'en'`

## TypeScript Types

```typescript
import type { Locale, I18nKey } from '$lib/i18n.svelte';
// Locale = 'en' | 'sv' | 'es' | ... (union of all supported codes)
// I18nKey = keyof typeof en.json (all valid keys)
```
