<script lang="ts">
	import { marketPlayer, progress } from '$lib/stores/marketPlayer';

	function handleSliderChange(event: Event) {
		const target = event.target as HTMLInputElement;
		const index = parseInt(target.value) - 1;
		marketPlayer.jumpToIndex(index);
	}
</script>

<div class="card p-4 space-y-3">
	<div class="flex items-center justify-between text-sm">
		<span class="font-semibold">
			Message {$progress.current.toLocaleString()} / {$progress.total.toLocaleString()}
		</span>
		<span class="text-surface-600-300-token">
			{$progress.percentage.toFixed(1)}%
		</span>
	</div>

	<!-- Progress Bar -->
	<input
		type="range"
		min="0"
		max={$progress.total}
		value={$progress.current}
		on:input={handleSliderChange}
		disabled={$progress.total === 0}
		class="w-full accent-primary-500"
	/>

	<!-- Loading Progress -->
	{#if $marketPlayer.isLoading}
		<div class="space-y-2">
			<div class="text-sm text-surface-600-300-token">
				Loading messages: {$marketPlayer.messages.length.toLocaleString()}
			</div>
			<div class="w-full bg-surface-300-600-token rounded-full h-2">
				<div
					class="bg-primary-500 h-2 rounded-full transition-all duration-300 animate-pulse"
					style="width: 100%"
				></div>
			</div>
		</div>
	{/if}
</div>
