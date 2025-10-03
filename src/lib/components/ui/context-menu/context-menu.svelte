<script lang="ts">
	import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
	import type { WithoutChild } from "$lib/utils.js";

	let {
		children,
		...restProps
	}: WithoutChild<ContextMenuPrimitive.RootProps> = $props();
</script>

<ContextMenuPrimitive.Root {...restProps}>
	{@render children?.()}
</ContextMenuPrimitive.Root>
