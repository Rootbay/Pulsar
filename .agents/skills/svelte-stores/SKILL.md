---
name: svelte-stores
description: How to create Svelte 5 rune-based stores using the class pattern with $state, $derived, $effect. Covers the project's store conventions and integration with the Rust backend.
---

# Creating Svelte 5 Rune Stores

This project exclusively uses Svelte 5 runes for reactivity. **Never use legacy `writable()`/`readable()` stores or `$:` syntax.**

## File Extension

Store files MUST use the `.svelte.ts` extension so the Svelte compiler processes runes:
```
src/lib/stores/myStore.svelte.ts
```

## Class-Based Store Pattern

```typescript
import { callBackend } from '../utils/backend';

class MyStore {
  // Reactive state
  items = $state<MyItem[]>([]);
  isLoading = $state(false);
  error = $state<string | null>(null);

  // Private reactive state (use # prefix)
  #searchTerm = $state('');

  // Derived values (automatically recompute)
  filteredItems = $derived(
    this.items.filter(item => item.name.includes(this.#searchTerm))
  );

  itemCount = $derived(this.items.length);

  // Complex derived with $derived.by()
  summary = $derived.by(() => {
    return `${this.itemCount} items, ${this.filteredItems.length} matching`;
  });

  // Constructor with effects
  constructor() {
    $effect.root(() => {
      // Effects that react to state changes
      $effect(() => {
        if (someCondition) {
          this.loadItems();
        }
      });
    });
  }

  // Getters for private state
  get searchTerm() {
    return this.#searchTerm;
  }

  set searchTerm(value: string) {
    this.#searchTerm = value;
  }

  // Async methods that call the Rust backend
  async loadItems() {
    this.isLoading = true;
    try {
      this.items = await callBackend<MyItem[]>('get_items');
    } catch (error) {
      console.error('Failed to load items:', error);
    } finally {
      this.isLoading = false;
    }
  }
}

// Export singleton instance
export const myStore = new MyStore();
```

## Key Rules

1. **Always use `callBackend()`** — never `invoke()` directly
2. **Use `$state.snapshot()`** when you need a non-reactive copy (e.g., for serialization)
3. **Use `$effect.root()`** in constructors to create effects outside component lifecycle
4. **Check `appState.isLocked`** before backend calls that need an unlocked vault
5. **Export as a singleton** — instantiate the class and export the instance

## Barrel Re-exports

After creating a new store, add it to `src/lib/stores.ts`:

```typescript
export { myStore } from './stores/myStore.svelte';
```

## Reference Stores

- `appState.svelte.ts` — Simple state with lock/unlock logic
- `vault.svelte.ts` — Complex store with search, pagination, CRUD
- `appSettings.svelte.ts` — Settings persistence with debounced save
- `sync.svelte.ts` — WebDAV sync with error handling
- `tags.svelte.ts` — Tag management with backend integration
