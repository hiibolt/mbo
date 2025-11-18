<script lang="ts">
	import { marketPlayer } from '$lib/stores/marketPlayer';
	import { formatPrice } from '$lib/types';
	import type { Book, Publisher } from '$lib/types';

	let expandedInstruments = new Set<number>();
	let expandedPublishers = new Set<string>();
	let expandedSides = new Set<string>();

	$: market = $marketPlayer.market;
	$: instrumentIds = Object.keys(market.books).map(Number);

	function toggleInstrument(id: number) {
		if (expandedInstruments.has(id)) {
			expandedInstruments.delete(id);
		} else {
			expandedInstruments.add(id);
		}
		expandedInstruments = expandedInstruments;
	}

	function togglePublisher(key: string) {
		if (expandedPublishers.has(key)) {
			expandedPublishers.delete(key);
		} else {
			expandedPublishers.add(key);
		}
		expandedPublishers = expandedPublishers;
	}

	function toggleSide(key: string) {
		if (expandedSides.has(key)) {
			expandedSides.delete(key);
		} else {
			expandedSides.add(key);
		}
		expandedSides = expandedSides;
	}

	function getPriceLevels(orders: Record<string, any[]>, side: 'bid' | 'ask'): Array<{ price: number; size: number; count: number }> {
		const levels = Object.entries(orders).map(([priceStr, orderList]) => {
			const price = Number(priceStr);
			const size = orderList.reduce((sum, order) => sum + (order.size || 0), 0);
			const count = orderList.length;
			return { price, size, count };
		});

		// Sort: bids descending (highest first), asks ascending (lowest first)
		return side === 'bid' 
			? levels.sort((a, b) => b.price - a.price)
			: levels.sort((a, b) => a.price - b.price);
	}
</script>

<div class="card p-4 space-y-4">
	<h3 class="h4 font-bold">Market State</h3>

	{#if instrumentIds.length === 0}
		<div class="text-center text-surface-600-300-token py-8">
			<p>No market data yet</p>
			<p class="text-sm">Load messages and start playback</p>
		</div>
	{:else}
		<div class="space-y-2 max-h-[600px] overflow-y-auto">
			{#each instrumentIds as instrumentId}
				{@const publishersWithBooks = market.books[instrumentId]}
				{@const isExpanded = expandedInstruments.has(instrumentId)}
				
				<div class="border border-surface-300-600-token rounded-lg">
					<!-- Instrument Header -->
					<button
						class="w-full p-3 flex items-center justify-between hover:bg-surface-100-800-token transition-colors"
						on:click={() => toggleInstrument(instrumentId)}
					>
						<div class="flex items-center gap-2">
							<span class="text-lg">{isExpanded ? '▼' : '▶'}</span>
							<span class="font-semibold">Instrument {instrumentId}</span>
						</div>
						<span class="text-sm text-surface-600-300-token">
							{publishersWithBooks.length} publisher{publishersWithBooks.length !== 1 ? 's' : ''}
						</span>
					</button>

					{#if isExpanded}
						<div class="border-t border-surface-300-600-token">
							{#each publishersWithBooks as [publisher, book], pubIdx}
								{@const pubKey = `${instrumentId}-${pubIdx}`}
								{@const isPubExpanded = expandedPublishers.has(pubKey)}
								{@const bidLevels = getPriceLevels(book.bids, 'bid')}
								{@const askLevels = getPriceLevels(book.offers, 'ask')}
								{@const bidKey = `${pubKey}-bids`}
								{@const askKey = `${pubKey}-asks`}
								{@const isBidExpanded = expandedSides.has(bidKey)}
								{@const isAskExpanded = expandedSides.has(askKey)}
								
								<!-- Publisher Section -->
								<div class="ml-6 border-l-2 border-surface-300-600-token">
									<button
										class="w-full p-3 flex items-center justify-between hover:bg-surface-100-800-token transition-colors"
										on:click={() => togglePublisher(pubKey)}
									>
										<div class="flex items-center gap-2">
											<span>{isPubExpanded ? '▼' : '▶'}</span>
											<span class="font-medium">{publisher}</span>
										</div>
										<span class="text-sm text-surface-600-300-token">
											{bidLevels.length} bids, {askLevels.length} asks
										</span>
									</button>

									{#if isPubExpanded}
										<div class="ml-6 space-y-2 p-3">
											<!-- Bids -->
											<button
												class="w-full flex items-center gap-2 p-2 rounded hover:bg-surface-100-800-token transition-colors"
												on:click={() => toggleSide(bidKey)}
											>
												<span>{isBidExpanded ? '▼' : '▶'}</span>
												<span class="font-medium text-green-600">Bids</span>
												<span class="text-sm text-surface-600-300-token">
													({bidLevels.length} levels)
												</span>
											</button>

											{#if isBidExpanded}
												<div class="ml-6 space-y-1">
													{#each bidLevels.slice(0, 10) as level}
														<div class="flex justify-between text-sm p-2 bg-green-50 dark:bg-green-900/20 rounded">
															<span class="font-mono">${formatPrice(level.price)}</span>
															<span>{level.size} size ({level.count} orders)</span>
														</div>
													{/each}
													{#if bidLevels.length > 10}
														<div class="text-xs text-surface-600-300-token ml-2">
															... and {bidLevels.length - 10} more levels
														</div>
													{/if}
												</div>
											{/if}

											<!-- Asks/Offers -->
											<button
												class="w-full flex items-center gap-2 p-2 rounded hover:bg-surface-100-800-token transition-colors"
												on:click={() => toggleSide(askKey)}
											>
												<span>{isAskExpanded ? '▼' : '▶'}</span>
												<span class="font-medium text-red-600">Asks/Offers</span>
												<span class="text-sm text-surface-600-300-token">
													({askLevels.length} levels)
												</span>
											</button>

											{#if isAskExpanded}
												<div class="ml-6 space-y-1">
													{#each askLevels.slice(0, 10) as level}
														<div class="flex justify-between text-sm p-2 bg-red-50 dark:bg-red-900/20 rounded">
															<span class="font-mono">${formatPrice(level.price)}</span>
															<span>{level.size} size ({level.count} orders)</span>
														</div>
													{/each}
													{#if askLevels.length > 10}
														<div class="text-xs text-surface-600-300-token ml-2">
															... and {askLevels.length - 10} more levels
														</div>
													{/if}
												</div>
											{/if}
										</div>
									{/if}
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>