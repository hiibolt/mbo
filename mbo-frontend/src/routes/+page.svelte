<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
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

	interface ErrorState {
		hasError: boolean;
		message: string;
		details?: string;
		canRetry: boolean;
	}

	// State
	let orderBook: OrderBook | null = null;
	let isLoading = true;
	let isConnected = false;
	let reconnectAttempts = 0;
	const MAX_RECONNECT_ATTEMPTS = 5;
	let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
	let eventSource: EventSource | null = null;

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

	// Connect to SSE stream
	function connectToStream() {
		try {
			isLoading = true;
			errorState = { hasError: false, message: '', canRetry: false };

			const url = '/api/mbo/stream/json';
			eventSource = new EventSource(url);

			eventSource.onopen = () => {
				isConnected = true;
				isLoading = false;
				reconnectAttempts = 0;
				showSuccessToast('Connected to order book stream');
			};

			eventSource.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data);
					orderBook = data;
				} catch (err) {
					const error = err as Error;
					console.error('Failed to parse SSE data:', error);
					showErrorToast('Data parsing error', error.message);
				}
			};

			eventSource.onerror = (event) => {
				isConnected = false;
				isLoading = false;

				if (eventSource?.readyState === EventSource.CLOSED) {
					handleDisconnection();
				} else {
					showErrorToast('Stream connection error', 'Connection unstable');
				}
			};
		} catch (err) {
			const error = err as Error;
			handleError(error, 'Failed to establish connection');
		}
	}

	// Handle disconnection with exponential backoff
	function handleDisconnection() {
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}

		if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
			reconnectAttempts++;
			const delay = Math.min(1000 * Math.pow(2, reconnectAttempts - 1), 30000);

			errorState = {
				hasError: true,
				message: `Connection lost. Reconnecting in ${(delay / 1000).toFixed(0)}s...`,
				details: `Attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS}`,
				canRetry: false
			};

			reconnectTimeout = setTimeout(() => {
				connectToStream();
			}, delay);
		} else {
			errorState = {
				hasError: true,
				message: 'Connection failed after multiple attempts',
				details: 'Please check the backend server and refresh the page',
				canRetry: true
			};
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

	// Manual retry
	function retryConnection() {
		reconnectAttempts = 0;
		if (reconnectTimeout) {
			clearTimeout(reconnectTimeout);
			reconnectTimeout = null;
		}
		connectToStream();
	}

	// Lifecycle
	onMount(() => {
		connectToStream();
	});

	onDestroy(() => {
		if (eventSource) {
			eventSource.close();
		}
		if (reconnectTimeout) {
			clearTimeout(reconnectTimeout);
		}
	});
</script>

<svelte:head>
	<title>MBO Order Book - Live Market Data</title>
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
						<button class="btn variant-filled" on:click={retryConnection}> Retry Connection </button>
					</div>
				{/if}
			</aside>
		{/if}

		<!-- Loading State -->
		{#if isLoading}
			<div class="card p-8 flex flex-col items-center justify-center space-y-4">
				<ProgressRadial width="w-32" />
				<h2 class="h2">Connecting to Order Book Stream...</h2>
				<p class="text-sm opacity-75">Establishing connection to backend server</p>
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
								Last Updated: {new Date(orderBook.timestamp).toLocaleString()}
							</p>
						</div>
						<div class="flex items-center gap-2">
							<span class="badge {isConnected ? 'variant-filled-success' : 'variant-filled-error'}">
								{isConnected ? '● Live' : '○ Disconnected'}
							</span>
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
								<span class="badge variant-soft-primary">Status</span>
								<span class="text-lg font-semibold">Active</span>
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
