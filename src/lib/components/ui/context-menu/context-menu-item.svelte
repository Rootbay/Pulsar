<script lang="ts">
  import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';
  import { cn, type WithoutChild } from '$lib/utils.js';

  let {
    ref = $bindable(null),
    class: className,
    inset = false,
    children,
    ...restProps
  }: WithoutChild<ContextMenuPrimitive.ItemProps> & {
    inset?: boolean;
  } = $props();
</script>

<ContextMenuPrimitive.Item
  bind:ref
  data-slot="context-menu-item"
  class={cn(
    'focus-visible:ring-ring data-highlighted:bg-accent data-highlighted:text-accent-foreground relative flex cursor-default items-center rounded-sm px-2 py-1.5 text-sm transition-colors outline-none select-none focus-visible:ring-1 focus-visible:outline-none data-disabled:pointer-events-none data-disabled:opacity-50',
    inset && 'pl-8',
    className
  )}
  {...restProps}
>
  {@render children?.()}
</ContextMenuPrimitive.Item>
