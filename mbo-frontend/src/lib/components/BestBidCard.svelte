<script lang="ts">
	export let bestBid: { price: number; size: number; count: number } | null;

	function formatPrice(price: number | null): string {
		if (price === null) return 'N/A';
		return (price / 1e9).toFixed(2);
	}
</script>

<div class="p-6">
	<header class="mb-4">
		<h3 class="text-2xl font-bold text-black">Best Bid</h3>
	</header>
	<section class="space-y-4">
		{#if bestBid}
			<div class="text-4xl font-bold text-black">
				${formatPrice(bestBid.price)}
			</div>
			<div class="grid grid-cols-2 gap-4 mt-6">
				<div class="bg-gray-50 rounded p-4">
					<div class="text-xs text-gray-500 font-semibold mb-1">SIZE</div>
					<div class="text-2xl font-bold text-black">{bestBid.size}</div>
				</div>
				<div class="bg-gray-50 rounded p-4">
					<div class="text-xs text-gray-500 font-semibold mb-1">ORDERS</div>
					<div class="text-2xl font-bold text-black">{bestBid.count}</div>
				</div>
			</div>
		{:else}
			<p class="text-center text-gray-400 py-8">No bid data available</p>
		{/if}
	</section>
</div>
