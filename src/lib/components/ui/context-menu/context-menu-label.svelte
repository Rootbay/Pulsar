<script lang="ts">
  import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';
  import { cn, type WithoutChild } from '$lib/utils.js';

  let {
    class: className,
    inset = false,
    children,
    ...restProps
  }: WithoutChild<ContextMenuPrimitive.GroupHeadingProps> & {
    inset?: boolean;
  } = $props();
</script>

<ContextMenuPrimitive.GroupHeading
  data-slot="context-menu-label"
  class={cn('text-muted-foreground px-2 py-1.5 text-xs font-semibold', inset && 'pl-8', className)}
  {...restProps}
>
  {@render children?.()}
</ContextMenuPrimitive.GroupHeading>
