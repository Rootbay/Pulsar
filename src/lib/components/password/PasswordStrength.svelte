<script lang="ts">
  import { SecurityService } from '$lib/utils/security';
  import { Progress } from '$lib/components/ui/progress';
  import { cn } from '$lib/utils';

  interface Props {
    password: string;
    showDetails?: boolean;
    isBreached?: boolean;
    className?: string;
  }

  let { password, showDetails = false, isBreached = false, className }: Props = $props();

  let result = $derived(SecurityService.checkStrength(password));
  let score = $derived(password ? (isBreached ? 0 : result.score) : -1);

  const STRENGTH_LABELS = ['Very Weak', 'Weak', 'Fair', 'Strong', 'Very Strong'];
  const STRENGTH_COLORS = [
    'bg-destructive',
    'bg-orange-500',
    'bg-yellow-500',
    'bg-emerald-500',
    'bg-emerald-500'
  ];

  const label = $derived(
    isBreached ? 'Breached' : score >= 0 ? STRENGTH_LABELS[score] : 'No Password'
  );
  const colorClass = $derived(
    isBreached ? 'bg-destructive' : score >= 0 ? STRENGTH_COLORS[score] : 'bg-muted'
  );
  const progressValue = $derived(isBreached ? 10 : score >= 0 ? (score + 1) * 20 : 0);
</script>

<div class={cn('space-y-1.5', className)}>
  <div class="flex items-center justify-between gap-2">
    <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase">
      Strength: <span class={cn('ml-1', score >= 0 ? colorClass.replace('bg-', 'text-') : '')}>
        {label}
      </span>
    </span>
    {#if showDetails && password && result.crackTimesDisplay}
      <span class="text-muted-foreground text-[10px]">
        Crack time: {result.crackTimesDisplay.offlineFastHashing1e10PerSecond}
      </span>
    {/if}
  </div>
  <Progress
    value={progressValue}
    class="bg-muted/40 h-1.5 w-full"
    indicatorClass={cn('transition-all duration-500', colorClass)}
  />
  {#if showDetails && score < 3 && result.feedback.suggestions.length > 0}
    <p class="text-muted-foreground mt-1 text-[10px] leading-relaxed">
      {result.feedback.warning ? result.feedback.warning + '. ' : ''}
      {result.feedback.suggestions[0]}
    </p>
  {/if}
</div>
