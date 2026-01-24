<script lang="ts">
</script>

<div class="setting-item">
  <slot name="info"></slot>
  <slot name="control"></slot>
</div>

<style>
  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-radius: var(--radius, 0.75rem);
    background-color: var(--setting-item-background-color);
    border: 1px solid var(--setting-item-border-color);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }
</style>
