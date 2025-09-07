<script lang="ts">
    import Icon from './Icon.svelte';

    export let icon: string;
    export let title: string;
    export let description: string;
</script>

<div class="card">
    <div class="card-header">
        <slot name="header-content">
            <Icon path={icon} size="24" color="currentColor" />
            <div>
                <h3>{title}</h3>
                <p>{description}</p>
            </div>
        </slot>
    </div>
    <slot></slot>
</div>

<style>
    :root {
        --background: 0 0% 5.9%;
        --foreground: 0 0% 98%;
        --card: 240 10% 3.9%;
        --card-foreground: 0 0% 98%;
        --primary: 262.1 83.3% 57.8%;
        --primary-foreground: 210 20% 98%;
        --secondary: 240 3.7% 15.9%;
        --muted: 240 3.7% 15.9%;
        --muted-foreground: 240 5% 64.9%;
        --border: 240 3.7% 15.9%;
        --input: 240 3.7% 15.9%;
        --ring: 262.1 83.3% 57.8%;
        --radius: 0.75rem;
        --shadow-card: 0 10px 30px -10px hsl(var(--background) / 0.3);
        --transition-smooth: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .card {
        background: linear-gradient(135deg, hsl(var(--card)), hsl(var(--muted) / 0.3));
        border: 1px solid hsl(var(--border) / 0.5);
        border-radius: var(--radius);
        padding: 1.5rem;
        transition: all 0.3s;
        margin: 0 0 32px 0;
    }
    .card:hover {
        border-color: hsl(var(--primary) / 0.3);
        box-shadow: 0 10px 30px -10px hsl(var(--primary) / 0.3);
    }
    .card-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-bottom: 1.25rem;
        padding-bottom: 1rem;
        border-bottom: 1px solid hsl(var(--border));
    }
    .card-header h3 {
        font-size: 1.125rem;
        font-weight: 600;
        margin: 0;
    }
    .card-header p {
        color: hsl(var(--muted-foreground));
        font-size: 0.875rem;
        margin: 0.125rem 0 0;
    }
</style>
