<script lang="ts">
	import { onMount } from 'svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';
	import type { ToastSettings } from '@skeletonlabs/skeleton';
	
	// Components
	import PlayerControls from '$lib/components/player/PlayerControls.svelte';
	import ProgressBar from '$lib/components/player/ProgressBar.svelte';
	import MessageInfo from '$lib/components/player/MessageInfo.svelte';
	import MarketTree from '$lib/components/market/MarketTree.svelte';
	
	// Services and Stores
	import { MarketStreamClient } from '$lib/services/streamClient';
	import { marketPlayer } from '$lib/stores/marketPlayer';
	
	const toastStore = getToastStore();
	
	let streamClient: MarketStreamClient | null = null;

	// Success toast helper
	function showSuccessToast(message: string) {
		const t: ToastSettings = {
			message,
			background: 'variant-filled-success',
			timeout: 3000
		};
		toastStore.trigger(t);
	}

	// Error toast helper
	function showErrorToast(message: string, details?: string) {
		const t: ToastSettings = {
			message: `<strong>Error:</strong> ${message}${details ? `<br><small>${details}</small>` : ''}`,
			background: 'variant-filled-error',
			timeout: 5000
		};
		toastStore.trigger(t);
	}

	async function loadMarketData() {
		try {
			marketPlayer.setLoading(true, 'connecting');
			streamClient = new MarketStreamClient();

			await streamClient.start(
				// Progress callback
				(progress) => {
					marketPlayer.setLoading(
						progress.status === 'streaming' || progress.status === 'connecting',
						progress.status
					);
					
					if (progress.status === 'streaming') {
						// Update message count in real-time
						const messages = streamClient!.getMessages();
						if (messages.length > 0) {
							marketPlayer.setMessages(messages);
						}
					}
					
					if (progress.error) {
						showErrorToast('Streaming error', progress.error);
					}
				},
				// Message callback
				(message) => {
					// Messages are batched in progress callback
				},
				// Complete callback
				(messages) => {
					marketPlayer.setMessages(messages);
					showSuccessToast(`Loaded ${messages.length.toLocaleString()} messages successfully!`);
				}
			);

		} catch (err) {
			const error = err as Error;
			marketPlayer.setError(error.message);
			showErrorToast('Failed to load market data', error.message);
		}
	}

	onMount(() => {
		loadMarketData();
	});
</script>

<svelte:head>
	<title>MBO Market Player</title>
</svelte:head>

<div class="min-h-screen bg-surface-50-900-token">
	<div class="container mx-auto p-8">
		<div class="w-full max-w-7xl mx-auto space-y-6">
			<!-- Header -->
			<!--
			<div class="card p-6">
				<h1 class="h1 font-bold mb-2">🎮 MBO Market Player</h1>
				<p class="text-surface-600-300-token">
					Real-time market order book playback and visualization
				</p>
			</div>
			-->

			<!-- Player Controls -->
			<PlayerControls />

			<!-- Progress Bar -->
			<ProgressBar />

			<!-- Main Content Grid -->
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
				<!-- Left: Market State -->
				<MarketTree />

				<!-- Right: Message Info -->
				<MessageInfo />
			</div>

			{#if $marketPlayer.error}
				<div class="alert variant-filled-error">
					<span>❌ Error: {$marketPlayer.error}</span>
				</div>
			{/if}
		</div>
	</div>
</div>
