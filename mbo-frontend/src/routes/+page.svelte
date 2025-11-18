<script lang="ts">
	import { onMount } from 'svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';
	import type { ToastSettings } from '@skeletonlabs/skeleton';
	
	// Components
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import LoadingState from '$lib/components/LoadingState.svelte';
	import HeaderCard from '$lib/components/HeaderCard.svelte';
	import BestBidCard from '$lib/components/BestBidCard.svelte';
	import BestOfferCard from '$lib/components/BestOfferCard.svelte';
	import MarketStatsCard from '$lib/components/MarketStatsCard.svelte';
	
	// Import types and utilities
	import type { 
		Market, 
		OrderBook, 
		ErrorState,
		PriceLevel 
	} from '$lib/types';
	import { 
		formatPrice, 
		calculateSpread, 
		calculateMidPrice 
	} from '$lib/types';
	
	const toastStore = getToastStore();

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

	// Calculate spread (wrapper for display)
	function getSpread(): string {
		if (!orderBook?.best_bid || !orderBook?.best_offer) return 'N/A';
		const spread = calculateSpread(orderBook.best_bid, orderBook.best_offer);
		return spread !== null ? formatPrice(spread) : 'N/A';
	}

	// Calculate mid price (wrapper for display)
	function getMidPrice(): string {
		if (!orderBook?.best_bid || !orderBook?.best_offer) return 'N/A';
		const mid = calculateMidPrice(orderBook.best_bid, orderBook.best_offer);
		return mid !== null ? formatPrice(mid) : 'N/A';
	}

	// Extract BBO from market data
	function extractOrderBook(market: Market): OrderBook | null {
		if (!market?.books) {
			console.error('Invalid market data structure');
			return null;
		}

		// Get the first instrument
		const instrumentIds = Object.keys(market.books);
		if (instrumentIds.length === 0) {
			console.error('No instruments found');
			return null;
		}

		const firstInstrument = market.books[instrumentIds[0]];
		if (!firstInstrument) return null;

		// Get the first publisher's book
		const publishers = Object.keys(firstInstrument);
		if (publishers.length === 0) return null;

		const book = firstInstrument[publishers[0]];
		if (!book) return null;

		// The book is an array: [venue_string, book_data]
		const bookData = Array.isArray(book) ? book[1] : book;

		// Calculate BBO (backend uses 'offers' not 'asks')
		const bids = bookData.bids || {};
		const asks = bookData.offers || bookData.asks || {};

		const bidPrices = Object.keys(bids).map(Number).sort((a, b) => b - a);
		const askPrices = Object.keys(asks).map(Number).sort((a, b) => a - b);

		let best_bid: PriceLevel | null = null;
		if (bidPrices.length > 0) {
			const price = bidPrices[0];
			const level = bids[price];
			let size = 0;
			let count = Object.keys(level).length;

			Object.values(level).forEach((order: any) => {
				size += order.size || 0;
			});

			best_bid = { price, size, count };
		}

		let best_offer: PriceLevel | null = null;
		if (askPrices.length > 0) {
			const price = askPrices[0];
			const level = asks[price];
			let size = 0;
			let count = Object.keys(level).length;

			Object.values(level).forEach((order: any) => {
				size += order.size || 0;
			});

			best_offer = { price, size, count };
		}

		return {
			symbol: 'CLX5', // Could extract from metadata if available
			timestamp: new Date().toISOString(),
			best_bid,
			best_offer
		};
	}

	// Fetch market snapshot
	async function fetchMarketSnapshot() {
		try {
			isLoading = true;
			errorState = { hasError: false, message: '', canRetry: false };

			const response = await fetch('/api/market/export');
			
			if (!response.ok) {
				throw new Error(`HTTP ${response.status}: ${response.statusText}`);
			}

			const market: Market = await response.json();
			orderBook = extractOrderBook(market);
			
			if (!orderBook) {
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

<div class="min-h-screen bg-white">
	<div class="container mx-auto p-8">
		<div class="w-full max-w-6xl mx-auto space-y-6">
			<!-- Error Banner -->
			{#if errorState.hasError}
				<ErrorBanner 
					message={errorState.message}
					details={errorState.details}
					canRetry={errorState.canRetry}
					onRetry={fetchMarketSnapshot}
				/>
			{/if}

			<!-- Loading State -->
			{#if isLoading}
				<LoadingState />
			{:else if orderBook}
				<!-- Order Book Display -->
				<div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
					<!-- Header -->
					<HeaderCard
						symbol={orderBook.symbol}
						lastUpdated={lastUpdated}
						onRefresh={fetchMarketSnapshot}
						onDownloadJSON={downloadJSON}
						onDownloadMBO={downloadMBOStream}
					/>

					<!-- Best Bid -->
					<BestBidCard bestBid={orderBook.best_bid} />

					<!-- Market Stats -->
					<MarketStatsCard 
						spread={getSpread()}
						midPrice={getMidPrice()}
					/>

					<!-- Best Offer -->
					<BestOfferCard bestOffer={orderBook.best_offer} />
				</div>
			{/if}
		</div>
	</div>
</div>
