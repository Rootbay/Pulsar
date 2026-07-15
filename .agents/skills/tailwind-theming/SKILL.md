---
name: tailwind-theming
description: How to use TailwindCSS v4's CSS-first theming in Pulsar with oklch colors, @theme inline, @custom-variant dark, and the design token system.
---

# TailwindCSS v4 Theming

Pulsar uses TailwindCSS v4's CSS-first configuration in `src/app.css`. No `tailwind.config.js` exists — everything is in CSS.

## Theme Architecture

### Design Tokens (`app.css`)

```css
@import 'tailwindcss';
@import 'tw-animate-css';
@custom-variant dark (&:is(.dark *));

:root {
  --radius: 0.625rem;
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);
  --primary: oklch(0.205 0 0);
  --primary-foreground: oklch(0.985 0 0);
  /* ... more tokens */
}

.dark {
  --background: oklch(0.145 0 0);
  --foreground: oklch(0.985 0 0);
  /* ... dark overrides */
}

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-primary: var(--primary);
  /* Maps CSS vars to Tailwind utilities */
}
```

### How It Works

1. **CSS Variables** (`:root` / `.dark`) define the raw oklch color values
2. **`@theme inline`** maps them to Tailwind color utilities (`bg-background`, `text-foreground`, etc.)
3. **Dark mode** toggles via `.dark` class on `<html>` element (managed by `+layout.svelte`)
4. **`@custom-variant dark`** enables the `dark:` prefix in Tailwind classes

## Available Design Tokens

| Token | Light | Dark | Utility |
|-------|-------|------|---------|
| `--background` | white | near-black | `bg-background` |
| `--foreground` | near-black | near-white | `text-foreground` |
| `--card` | white | `#0f0f0f` | `bg-card` |
| `--primary` | dark | light | `bg-primary` |
| `--secondary` | light gray | dark gray | `bg-secondary` |
| `--muted` | light gray | dark gray | `bg-muted` |
| `--muted-foreground` | medium gray | medium gray | `text-muted-foreground` |
| `--accent` | light gray | dark gray | `bg-accent` |
| `--destructive` | red | red | `bg-destructive` |
| `--border` | light gray | white/10% | `border-border` |
| `--input` | light gray | white/15% | `border-input` |
| `--ring` | medium gray | medium gray | `ring-ring` |
| `--sidebar` | near-white | `#121212` | `bg-sidebar` |

### Panel-Specific Tokens
- `--sidebar-width`: sidebar panel width (default: `16rem`)
- `--passwordList-width`: password list panel width (default: `320px`)
- `--passwordlist-base`, `--passwordlist-item`, `--passwordlist-hover-bg`: list colors

## Accessibility Variants

```css
.high-contrast {
  --border: oklch(0 0 0);    /* Pure black borders */
}
.dark.high-contrast {
  --border: oklch(1 0 0);    /* Pure white borders */
}
.reduced-motion * {
  animation-duration: 0.001ms !important;
}
.density-compact { --spacing: 0.2rem; }
.density-dense { --spacing: 0.15rem; }
.compact-mode { --radius: 0.25rem; }
```

## Adding New Theme Tokens

1. Add CSS variable to both `:root` and `.dark` in `app.css`
2. Map to Tailwind in `@theme inline`: `--color-my-token: var(--my-token);`
3. Use in components: `bg-my-token`, `text-my-token`, etc.

## Rules

- **ALWAYS** use design tokens (`bg-background`) — never hardcode colors like `bg-white`
- Exception: One-off accent colors that don't need dark-mode awareness
- Use `oklch()` for all new color values — it's perceptually uniform
- The theme is managed by the `AppearanceSettings` in the settings system
