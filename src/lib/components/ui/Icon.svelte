<script lang="ts">
  interface Props {
    path: string;
    color?: string;
    size?: string | number;
    stroke?: string;
    strokeWidth?: number | string;
    transform?: string;
    className?: string;
    viewBox?: string;
  }

  let {
    path,
    color = "currentColor",
    size = "24",
    stroke = undefined,
    strokeWidth = undefined,
    transform = undefined,
    className = '',
    viewBox = "0 0 44 44",
  }: Props = $props();

  // Compute CSS size string: add 'px' when numeric, otherwise passthrough (e.g., '1em')
  const styleSize = $derived(() => {
    const s = String(size);
    return /^(\d+(?:\.\d+)?)$/.test(s) ? `${s}px` : s;
  });
</script>

<svg class={className}
  viewBox={viewBox}
  width={size}
  height={size}
  style={`color: ${color}; width: ${styleSize}; height: ${styleSize}; display: block;`}
  fill="currentColor"
  stroke={stroke}
  stroke-width={strokeWidth}
  preserveAspectRatio="xMidYMid meet"
>
  <path d={path} transform={transform}></path>
</svg>
