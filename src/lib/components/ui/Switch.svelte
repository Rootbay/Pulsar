<script lang="ts">
	export let checked: boolean = false;
	export let ariaLabel: string = 'Toggle switch';
	export let id: string = '';
</script>

<button
	id={id}
	role="switch"
	aria-checked={checked}
	aria-label={ariaLabel}
	class="switch"
	class:active={checked}
	on:click
>
	<span class="slider"></span>
</button>

<style>
	.switch {
		position: relative;
		display: inline-block;
		width: 44px;
		height: 24px;
		background-color: hsl(var(--input));
		border-radius: 12px;
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		border: none;
		padding: 0;
		flex-shrink: 0;
	}

	.switch.active {
		background-color: hsl(var(--primary));
	}

	.slider {
		position: absolute;
		cursor: pointer;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: transparent;
		transition: .4s;
		border-radius: 24px;
	}

	.slider:before {
		position: absolute;
		content: "";
		height: 20px;
		width: 20px;
		left: 2px;
		bottom: 2px;
		background-color: hsl(var(--foreground));
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		border-radius: 50%;
		box-shadow: 0 1px 3px rgba(0,0,0,0.2);
	}

	.switch.active .slider:before {
		transform: translateX(20px);
	}
</style>