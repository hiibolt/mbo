<script lang="ts">
	export let symbol: string;
	export let lastUpdated: Date | null;
	export let onRefresh: () => void;
	export let onDownloadJSON: () => void;
	export let onDownloadMBO: () => void;
</script>

<div class="lg:col-span-3 p-6">
	<div class="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-4">
		<div>
			<h1 class="text-5xl font-bold text-black tracking-tight">{symbol}</h1>
			<p class="text-sm text-gray-600 mt-2">
				Last Updated: {lastUpdated ? lastUpdated.toLocaleString() : 'N/A'}
			</p>
		</div>
		<div class="flex items-center gap-3 flex-wrap">
			<button
				class="px-5 py-2.5 bg-white border-2 border-black text-black hover:bg-gray-50 rounded font-semibold transition-all duration-200"
				on:click={onRefresh}
				title="Refresh snapshot"
			>
				Refresh
			</button>
			<button
				class="px-5 py-2.5 bg-white border-2 border-black text-black hover:bg-gray-50 rounded font-semibold transition-all duration-200"
				on:click={onDownloadJSON}
				title="Download full market data as JSON"
			>
				JSON
			</button>
			<button
				class="px-5 py-2.5 bg-white border-2 border-black text-black hover:bg-gray-50 rounded font-semibold transition-all duration-200"
				on:click={onDownloadMBO}
				title="Download all MBO messages"
			>
				MBO
			</button>
		</div>
	</div>
</div>
