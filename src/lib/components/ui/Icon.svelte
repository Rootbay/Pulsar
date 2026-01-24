<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    path?: string;
    size?: string | number;
    color?: string;
    viewBox?: string;
    stroke?: string;
    strokeWidth?: number;
    className?: string;
    children?: Snippet;
    [key: string]: any;
  }

  let {
    path = '',
    size = '24',
    color = 'currentColor',
    viewBox = '0 0 24 24',
    stroke = undefined,
    strokeWidth = undefined,
    className = '',
    children,
    ...rest
  }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  {viewBox}
  fill={stroke ? 'none' : color}
  stroke={stroke || (strokeWidth ? color : 'none')}
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  {...rest}
>
  {#if path}
    <path d={path} />
  {/if}
  {@render children?.()}
</svg>
