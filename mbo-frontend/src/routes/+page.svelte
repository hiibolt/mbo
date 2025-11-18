<script lang="ts">
	import { onMount } from 'svelte';
	import { ProgressRadial } from '@skeletonlabs/skeleton';
	import { getToastStore } from '@skeletonlabs/skeleton';
	import type { ToastSettings } from '@skeletonlabs/skeleton';

	const toastStore = getToastStore();

	// Types
	interface PriceLevel {
		price: number;
		size: number;
		count: number;
	}

	interface OrderBook {
		symbol: string;
		timestamp: string;
		best_bid: PriceLevel | null;
		best_offer: PriceLevel | null;
	}

	interface Market {
		books: Record<string, any>;
	}

	interface ErrorState {
		hasError: boolean;
		message: string;
		details?: string;
		canRetry: boolean;
	}

	// State
	let orderBook: OrderBook | null = null;
	let isLoading = true;
	let lastUpdated: Date | null = null;

	let errorState: ErrorState = {
		hasError: false,
		message: '',
		canRetry: false
	};

	// Error toast helper
	function showErrorToast(message: string, details?: string) {
		const t: ToastSettings = {
			message: `<strong>Error:</strong> ${message}${details ? `<br><small>${details}</small>` : ''}`,
			background: 'variant-filled-error',
			timeout: 5000
		};
		toastStore.trigger(t);
	}

	// Success toast helper
	function showSuccessToast(message: string) {
		const t: ToastSettings = {
			message,
			background: 'variant-filled-success',
			timeout: 3000
		};
		toastStore.trigger(t);
	}

	// Format price for display
	function formatPrice(price: number | null): string {
		if (price === null) return 'N/A';
		return (price / 1e9).toFixed(2);
	}

	// Calculate spread
	function calculateSpread(): string {
		if (!orderBook?.best_bid || !orderBook?.best_offer) return 'N/A';
		const spread = orderBook.best_offer.price - orderBook.best_bid.price;
		return (spread / 1e9).toFixed(2);
	}

	// Calculate mid price
	function calculateMidPrice(): string {
		if (!orderBook?.best_bid || !orderBook?.best_offer) return 'N/A';
		const mid = (orderBook.best_bid.price + orderBook.best_offer.price) / 2;
		return (mid / 1e9).toFixed(2);
	}

	// Extract BBO from market data
	function extractOrderBook(market: Market): OrderBook | null {
		console.log('Extracting order book from market data:', market);

		if (!market) {
			console.error('Market is null or undefined');
			return null;
		}

		if (!market.books) {
			console.error('Market.books is missing. Market structure:', Object.keys(market));
			return null;
		}

		// Get the first instrument
		const instrumentIds = Object.keys(market.books);
		console.log('Found instruments:', instrumentIds);
		
		if (instrumentIds.length === 0) {
			console.error('No instruments found in market.books');
			return null;
		}

		const firstInstrument = market.books[instrumentIds[0]];
		console.log('First instrument data:', firstInstrument);
		
		if (!firstInstrument) {
			console.error('First instrument is null');
			return null;
		}

		// Get the first publisher's book
		const publishers = Object.keys(firstInstrument);
		console.log('Found publishers:', publishers);
		
		if (publishers.length === 0) {
			console.error('No publishers found for instrument');
			return null;
		}

		const book = firstInstrument[publishers[0]];
		console.log('Book data:', book);
		console.log('Book keys:', Object.keys(book));
		console.log('Book full structure:', JSON.stringify(book, null, 2));
		
		if (!book) {
			console.error('Book is null');
			return null;
		}

		// The book is an array: [venue_string, book_data]
		// We need book[1] to get the actual book data
		const bookData = Array.isArray(book) ? book[1] : book;
		console.log('Actual book data:', bookData);

		// Calculate BBO
		// Note: backend uses 'offers' not 'asks'
		const bids = bookData.bids || {};
		const asks = bookData.offers || bookData.asks || {};
		
		console.log('Bids:', bids);
		console.log('Offers/Asks:', asks);

		const bidPrices = Object.keys(bids)
			.map(Number)
			.sort((a, b) => b - a);
		const askPrices = Object.keys(asks)
			.map(Number)
			.sort((a, b) => a - b);

		console.log('Bid prices (sorted):', bidPrices);
		console.log('Ask prices (sorted):', askPrices);

		let best_bid = null;
		if (bidPrices.length > 0) {
			const price = bidPrices[0];
			const level = bids[price];
			let size = 0;
			let count = Object.keys(level).length;
			Object.values(level).forEach((order: any) => {
				size += order.size || 0;
			});
			best_bid = { price, size, count };
			console.log('Best bid:', best_bid);
		} else {
			console.warn('No bids available');
		}

		let best_offer = null;
		if (askPrices.length > 0) {
			const price = askPrices[0];
			const level = asks[price];
			console.log('Processing best offer - price:', price, 'level:', level);
			let size = 0;
			let count = Object.keys(level).length;
			Object.values(level).forEach((order: any) => {
				size += order.size || 0;
			});
			best_offer = { price, size, count };
			console.log('Best offer:', best_offer);
		} else {
			console.warn('No offers available');
		}

		const result = {
			symbol: 'CLX5', // Could extract from metadata if available
			timestamp: new Date().toISOString(),
			best_bid,
			best_offer
		};

		console.log('Final order book:', result);
		return result;
	}

	// Fetch market snapshot
	async function fetchMarketSnapshot() {
		try {
			isLoading = true;
			errorState = { hasError: false, message: '', canRetry: false };

			console.log('Fetching market snapshot from /api/market/export...');
			const response = await fetch('/api/market/export');
			
			console.log('Response status:', response.status, response.statusText);
			console.log('Response headers:', Object.fromEntries(response.headers.entries()));
			
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}: ${response.statusText}`);
			}

			const rawText = await response.text();
			console.log('Raw response (first 500 chars):', rawText.substring(0, 500));

			let market: Market;
			try {
				market = JSON.parse(rawText);
				console.log('Parsed market data:', market);
				console.log('Market data structure:', {
					hasBooks: !!market.books,
					bookKeys: market.books ? Object.keys(market.books) : 'N/A',
					topLevelKeys: Object.keys(market)
				});
			} catch (parseErr) {
				console.error('JSON parse error:', parseErr);
				console.error('Failed to parse response:', rawText);
				throw new Error(`Failed to parse JSON response: ${parseErr}`);
			}

			orderBook = extractOrderBook(market);
			
			if (!orderBook) {
				console.error('extractOrderBook returned null');
				throw new Error('Failed to extract order book from market data');
			}
			
			lastUpdated = new Date();
			isLoading = false;

			showSuccessToast('Market snapshot loaded successfully');
		} catch (err) {
			const error = err as Error;
			handleError(error, 'Failed to fetch market snapshot');
		}
	}

	// Generic error handler
	function handleError(error: Error, context: string) {
		console.error(`Error in ${context}:`, error);
		errorState = {
			hasError: true,
			message: context,
			details: error.message,
			canRetry: true
		};
		isLoading = false;
		showErrorToast(context, error.message);
	}

	// Download JSON export
	async function downloadJSON() {
		try {
			const response = await fetch('/api/market/export');
			if (!response.ok) throw new Error(`HTTP ${response.status}`);

			const blob = await response.blob();
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `market_export_${new Date().toISOString()}.json`;
			document.body.appendChild(a);
			a.click();
			window.URL.revokeObjectURL(url);
			document.body.removeChild(a);

			showSuccessToast('JSON export downloaded');
		} catch (err) {
			const error = err as Error;
			showErrorToast('Download failed', error.message);
		}
	}

	// Download binary stream (all MBO messages)
	async function downloadMBOStream() {
		try {
			showSuccessToast('Streaming MBO messages... This may take a moment.');
			
			const response = await fetch('/api/mbo/stream/json');
			if (!response.ok) throw new Error(`HTTP ${response.status}`);

			const reader = response.body?.getReader();
			if (!reader) throw new Error('No response body');

			const chunks: Uint8Array[] = [];
			while (true) {
				const { done, value } = await reader.read();
				if (done) break;
				chunks.push(value);
			}

			// Combine chunks
			const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
			const combined = new Uint8Array(totalLength);
			let offset = 0;
			for (const chunk of chunks) {
				combined.set(chunk, offset);
				offset += chunk.length;
			}

			const blob = new Blob([combined], { type: 'text/plain' });
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `mbo_messages_${new Date().toISOString()}.jsonl`;
			document.body.appendChild(a);
			a.click();
			window.URL.revokeObjectURL(url);
			document.body.removeChild(a);

			showSuccessToast('MBO stream downloaded successfully');
		} catch (err) {
			const error = err as Error;
			showErrorToast('Stream download failed', error.message);
		}
	}

	// Lifecycle
	onMount(() => {
		fetchMarketSnapshot();
	});
</script>

<svelte:head>
	<title>MBO Order Book - Market Snapshot</title>
</svelte:head>

<div class="container h-full mx-auto flex justify-center items-center p-4">
	<div class="w-full max-w-6xl space-y-4">
		<!-- Connection Status Banner -->
		{#if errorState.hasError}
			<aside class="alert variant-filled-error">
				<div class="alert-message">
					<h3 class="h3">⚠️ {errorState.message}</h3>
					{#if errorState.details}
						<p class="text-sm mt-2">{errorState.details}</p>
					{/if}
				</div>
				{#if errorState.canRetry}
					<div class="alert-actions">
						<button class="btn variant-filled" on:click={fetchMarketSnapshot}> Retry </button>
					</div>
				{/if}
			</aside>
		{/if}

		<!-- Loading State -->
		{#if isLoading}
			<div class="card p-8 flex flex-col items-center justify-center space-y-4">
				<ProgressRadial width="w-32" />
				<h2 class="h2">Loading Market Snapshot...</h2>
				<p class="text-sm opacity-75">Fetching order book data from backend</p>
			</div>
		{:else if orderBook}
			<!-- Order Book Display -->
			<div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
				<!-- Header Card -->
				<div class="lg:col-span-3 card p-6 variant-filled-surface">
					<div class="flex justify-between items-center">
						<div>
							<h1 class="h1">{orderBook.symbol}</h1>
							<p class="text-sm opacity-75">
								Last Updated: {lastUpdated ? lastUpdated.toLocaleString() : 'N/A'}
							</p>
						</div>
						<div class="flex items-center gap-2">
							<button
								class="btn variant-filled-primary"
								on:click={fetchMarketSnapshot}
								title="Refresh snapshot"
							>
								🔄 Refresh
							</button>
							<button
								class="btn variant-filled-secondary"
								on:click={downloadJSON}
								title="Download full market data as JSON"
							>
								📥 JSON
							</button>
							<button
								class="btn variant-filled-tertiary"
								on:click={downloadMBOStream}
								title="Download all MBO messages"
							>
								📊 MBO Stream
							</button>
						</div>
					</div>
				</div>

				<!-- Best Bid -->
				<div class="card p-6 variant-ghost-success">
					<header class="card-header">
						<h3 class="h3">Best Bid</h3>
					</header>
					<section class="p-4 space-y-2">
						{#if orderBook.best_bid}
							<div class="text-4xl font-bold text-success-500">
								${formatPrice(orderBook.best_bid.price)}
							</div>
							<dl class="list-dl">
								<div>
									<span class="badge variant-soft-success">Size</span>
									<span>{orderBook.best_bid.size}</span>
								</div>
								<div>
									<span class="badge variant-soft-success">Orders</span>
									<span>{orderBook.best_bid.count}</span>
								</div>
							</dl>
						{:else}
							<p class="text-center opacity-50">No bid data available</p>
						{/if}
					</section>
				</div>

				<!-- Market Stats -->
				<div class="card p-6 variant-ghost-primary">
					<header class="card-header">
						<h3 class="h3">Market Stats</h3>
					</header>
					<section class="p-4 space-y-2">
						<dl class="list-dl">
							<div>
								<span class="badge variant-soft-primary">Spread</span>
								<span class="text-lg font-semibold">${calculateSpread()}</span>
							</div>
							<div>
								<span class="badge variant-soft-primary">Mid Price</span>
								<span class="text-lg font-semibold">${calculateMidPrice()}</span>
							</div>
							<div>
								<span class="badge variant-soft-primary">Type</span>
								<span class="text-lg font-semibold">Snapshot</span>
							</div>
						</dl>
					</section>
				</div>

				<!-- Best Offer -->
				<div class="card p-6 variant-ghost-error">
					<header class="card-header">
						<h3 class="h3">Best Offer</h3>
					</header>
					<section class="p-4 space-y-2">
						{#if orderBook.best_offer}
							<div class="text-4xl font-bold text-error-500">
								${formatPrice(orderBook.best_offer.price)}
							</div>
							<dl class="list-dl">
								<div>
									<span class="badge variant-soft-error">Size</span>
									<span>{orderBook.best_offer.size}</span>
								</div>
								<div>
									<span class="badge variant-soft-error">Orders</span>
									<span>{orderBook.best_offer.count}</span>
								</div>
							</dl>
						{:else}
							<p class="text-center opacity-50">No offer data available</p>
						{/if}
					</section>
				</div>
			</div>
		{/if}
	</div>
</div>
