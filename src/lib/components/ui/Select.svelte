<script lang="ts">
  interface Props {
    value: string;
    options: { value: string; label: string }[];
    ariaLabel?: string;
    onchange?: (event: Event) => void;
  }

  let { value = $bindable(), options, ariaLabel = 'Select option', onchange }: Props = $props();
</script>

<select class="select" bind:value {onchange} aria-label={ariaLabel}>
  {#each options as option (option.value)}
    <option value={option.value}>{option.label}</option>
  {/each}
</select>

<style>
  .select {
    background: hsl(var(--secondary));
    border: 1px solid hsl(var(--border));
    border-radius: var(--radius);
    padding: 0.5rem 2.5rem 0.5rem 1rem;
    color: hsl(var(--foreground));
    font-size: 0.875rem;
    min-width: 120px;
    cursor: pointer;
    -webkit-appearance: none;
    appearance: none;
    background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3e%3cpath fill='none' stroke='%239ca3af' stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M2 5l6 6 6-6'/%3e%3c/svg%3e");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
    background-size: 1em;
    transition: var(--transition-smooth);
  }

  .select:hover {
    border-color: hsl(var(--primary) / 0.5);
  }

  .select:focus {
    outline: none;
    border-color: hsl(var(--ring));
    box-shadow: 0 0 0 3px hsl(var(--ring) / 0.2);
  }
</style>
