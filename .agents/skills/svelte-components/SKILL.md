---
name: svelte-components
description: How to create Svelte 5 components using runes mode, TailwindCSS v4 utilities, shadcn-svelte primitives, lucide icons, and the project's i18n system.
---

# Creating Svelte 5 Components

## Component Template

```svelte
<script lang="ts">
  import { cn } from '$lib/utils';
  import { t } from '$lib/i18n.svelte';
  import { i18n } from '$lib/i18n.svelte';
  import { SomeIcon } from '$lib/icons';
  import { Button } from '$lib/components/ui/button';
  import { callBackend } from '$lib/utils/backend';

  // Props using $props() — never use export let
  let {
    title,
    onSave,
    class: className,
  }: {
    title: string;
    onSave?: (value: string) => void;
    class?: string;
  } = $props();

  // Local reactive state
  let inputValue = $state('');
  let isLoading = $state(false);

  // Derived values
  const isValid = $derived(inputValue.trim().length > 0);

  // Locale for i18n
  const locale = $derived(i18n.locale);

  // Methods
  async function handleSave() {
    isLoading = true;
    try {
      await callBackend('save_item', { value: inputValue });
      onSave?.(inputValue);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class={cn('flex flex-col gap-2 p-4', className)}>
  <h2 class="text-lg font-semibold text-foreground">
    {t(locale, 'some_key')}
  </h2>

  <input
    type="text"
    bind:value={inputValue}
    class="rounded-md border border-input bg-background px-3 py-2 text-sm"
    placeholder={t(locale, 'placeholder_key')}
  />

  <Button
    onclick={handleSave}
    disabled={!isValid || isLoading}
  >
    <SomeIcon class="mr-2 h-4 w-4" />
    {t(locale, 'save_button')}
  </Button>
</div>
```

## Key Rules

### Props
- Use `$props()` with destructuring and TypeScript types — **never `export let`**
- Use `class: className` pattern for forwarding CSS classes
- Use callback props (`onSave`) instead of Svelte events

### Styling
- Use **TailwindCSS utilities** — never custom CSS unless absolutely necessary
- Use `cn()` from `$lib/utils.ts` for conditional class merging
- Use design tokens: `text-foreground`, `bg-background`, `border-border`, `text-muted-foreground`
- Dark mode works via `.dark` class on `<html>` — use `dark:` variant when needed

### UI Primitives
- Use shadcn-svelte components from `$lib/components/ui/`:
  - `button`, `input`, `dialog`, `sheet`, `card`, `badge`, `checkbox`, `switch`, `select`, `tooltip`, `scroll-area`, `separator`, `avatar`, `skeleton`, `spinner`, `alert`, `alert-dialog`, `context-menu`, `progress`, `label`
- Use custom wrappers: `Popup`, `FieldInput`, `Select`, `Switch`, `SettingItem`, `SettingsCard`, `EditModal`

### Icons
- Import from `$lib/icons.ts` which barrel-exports lucide icons
- Standard sizing: `class="h-4 w-4"` for inline, `class="h-5 w-5"` for buttons

### i18n
- All user-facing strings MUST use `t(locale, 'key')` — never hardcode text
- Get locale: `const locale = $derived(i18n.locale);`
- Add translation keys to all 20 JSON files in `src/lib/i18n/`

### Naming
- Components: **PascalCase** (e.g., `MyButton.svelte`)
- File location: `src/lib/components/` or appropriate subdirectory
- Popup components: suffix with `Popup` (e.g., `CreatePasswordPopup.svelte`)

### Snippets (Svelte 5)
- Use `{@render children()}` for slot content — not `<slot>`
- Use `{#snippet name()}...{/snippet}` for named snippets
