<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils.js';

  let {
    class: className,
    children,
    ...restProps
  }: {
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<span
  data-slot="context-menu-shortcut"
  class={cn('text-muted-foreground/70 ml-auto text-xs tracking-widest', className)}
  {...restProps}
>
  {@render children?.()}
</span>
