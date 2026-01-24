<script lang="ts">
  export let path: string = "";
  export let size: string | number = "24";
  export let color: string = "currentColor";
  export let viewBox: string = "0 0 24 24";
  export let stroke: string | undefined = undefined;
  export let strokeWidth: number | undefined = undefined;
  export let className: string = "";
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox={viewBox}
  fill={stroke ? "none" : color}
  stroke={stroke || (strokeWidth ? color : "none")}
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  {...$$restProps}
>
  {#if path}
    <path d={path} />
  {/if}
  <slot />
</svg>
