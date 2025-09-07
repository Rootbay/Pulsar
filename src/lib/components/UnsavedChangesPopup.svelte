<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { fade } from 'svelte/transition';

	const dispatch = createEventDispatcher();

	function onReset() {
		dispatch('reset');
	}

	function onSave() {
		dispatch('save');
	}
</script>

<div class="unsaved-changes-popup" transition:fade={{ duration: 150 }}>
	<span>Careful - you have unsaved changes!</span>
	<div class="save-actions">
		<button class="btn btn-outline" on:click={onReset}>Reset</button>
		<button class="btn btn-primary" on:click={onSave}>Save Changes</button>
	</div>
</div>

<style>
	.unsaved-changes-popup {
		position: fixed;
		bottom: 20px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: calc(100% - 40px);
		max-width: 600px;
		height: 52px;
		color: white;
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
		z-index: 200;
		padding: 1rem;
	  	background: hsl(var(--card) / 0.95);
	  	border-radius: calc(var(--radius));
	  	border: 1px solid hsl(var(--border));
	}
    
	

	.save-actions {
        display: flex;
        gap: 0.75rem;
    }

	.btn-outline {
        background: transparent;
        border: 1px solid hsl(var(--border));
        color: hsl(var(--foreground));
    }

    .btn-outline:hover {
        background: hsl(var(--accent));
    }

    .btn-primary {
        background: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
        border: 1px solid hsl(var(--primary));
    }

    .btn-primary:hover {
        background: hsl(var(--primary) / 0.9);
    }
	
    .btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        border-radius: calc(var(--radius));
        font-size: 0.875rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
        text-decoration: none;
        border: none;
    }

	@keyframes pulse {
        0%, 100% {
            opacity: 1;
        }
        50% {
            opacity: 0.5;
        }
    }
</style>
