<script lang="ts">
  import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';
  import { cn, type WithoutChild } from '$lib/utils.js';

  let { class: className, ...restProps }: WithoutChild<ContextMenuPrimitive.SeparatorProps> =
    $props();
</script>

<ContextMenuPrimitive.Separator
  data-slot="context-menu-separator"
  class={cn('bg-border/60 -mx-1 my-1 h-px', className)}
  {...restProps}
/>
