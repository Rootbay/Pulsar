<script lang="ts">
  export let path: string;
  export let color: string = "currentColor";
  export let size: string = "24";
  export let stroke: string | undefined = undefined;
  export let transform: string | undefined = undefined;
  export let className: string = '';
  
  export let viewBox: string = "0 0 48 48";
</script>

<svg class={className}
  viewBox={viewBox}
  width={size}
  height={size}
  fill={color}
  stroke={stroke}
>
  <path d={path} transform={transform}></path>
</svg>
