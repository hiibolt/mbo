<script lang="ts">
	export let message: string;
	export let details: string | undefined = undefined;
	export let canRetry: boolean = false;
	export let onRetry: (() => void) | undefined = undefined;
</script>

<aside class="bg-red-50 p-6 rounded-lg">
	<div class="flex items-start justify-between gap-4">
		<div class="flex-1">
			<h3 class="text-lg font-semibold text-red-600 flex items-center gap-2">
				⚠️ {message}
			</h3>
			{#if details}
				<p class="text-sm mt-2 text-gray-600">{details}</p>
			{/if}
		</div>
		{#if canRetry && onRetry}
			<button 
				class="px-4 py-2 bg-white border-2 border-black text-black hover:bg-gray-50 rounded font-medium transition-colors whitespace-nowrap" 
				on:click={onRetry}
			>
				Retry
			</button>
		{/if}
	</div>
</aside>
