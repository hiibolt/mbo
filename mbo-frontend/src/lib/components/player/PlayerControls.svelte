<script lang="ts">
	import { marketPlayer, canPlayPause, canGoBack, canGoForward } from '$lib/stores/marketPlayer';

	let playbackSpeed = 1;
	const speeds = [1, 2, 5, 10];

	function handleSpeedChange() {
		marketPlayer.setPlaybackSpeed(playbackSpeed);
	}
	// Download JSON export
	export async function downloadJSON() {
		try {
			const response = await fetch('/api/market/export');

			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`);
			}

			const filename = `market_export_${new Date().toISOString()}.zip`;

			// If the browser supports streaming:
			if (response.body) {
				// Create a stream that pulls from the network and pushes into a Blob
				const reader = response.body.getReader();
				const chunks: BlobPart[] = [];

				// Stream the data manually
				while (true) {
					const { done, value } = await reader.read();
					if (done) break;
					chunks.push(value);
				}

				// Build a blob the browser can download
				const blob = new Blob(chunks, { type: 'application/zip' });
				const url = URL.createObjectURL(blob);

				const a = document.createElement('a');
				a.href = url;
				a.download = filename;
				document.body.appendChild(a);
				a.click();
				document.body.removeChild(a);
				URL.revokeObjectURL(url);
				return;
			}

			// Fallback for ancient browsers
			const blob = await response.blob();
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = filename;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

		} catch (err) {
			console.error("Download failed:", (err as Error).message);
		}
	}
</script>

<div class="card p-4 space-y-4">
   <div class="flex items-center justify-between gap-4">
	   <!-- Direct Download JSON Export -->
	   <a
		   class="px-5 py-2.5 bg-white border-2 border-black text-black hover:bg-gray-50 rounded font-semibold transition-all duration-200"
		   href="/api/market/export"
		   download="market_export.zip"
		   title="Download full market data as JSON"
	   >
		   JSON
	   </a>

		<!-- Jump to Start -->
		<button
			class="btn btn-sm variant-filled-surface"
			on:click={() => marketPlayer.jumpToStart()}
			disabled={!$canGoBack}
			title="Jump to start"
		>
			<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
				<path d="M8.445 14.832A1 1 0 0010 14v-2.798l5.445 3.63A1 1 0 0017 14V6a1 1 0 00-1.555-.832L10 8.798V6a1 1 0 00-1.555-.832l-6 4a1 1 0 000 1.664l6 4z" />
			</svg>
		</button>

		<!-- Previous -->
		<button
			class="btn btn-sm variant-filled-surface"
			on:click={() => marketPlayer.previous()}
			disabled={!$canGoBack}
			title="Previous message"
		>
			<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
				<path d="M8.445 14.832A1 1 0 0010 14V6a1 1 0 00-1.555-.832l-6 4a1 1 0 000 1.664l6 4zM14.555 14.832A1 1 0 0016 14V6a1 1 0 00-1.555-.832l-6 4a1 1 0 000 1.664l6 4z" />
			</svg>
		</button>

		<!-- Play/Pause -->
		<button
			class="btn btn-lg variant-filled-primary"
			on:click={() => $marketPlayer.isPlaying ? marketPlayer.pause() : marketPlayer.play()}
			disabled={!$canPlayPause && !$marketPlayer.isPlaying}
			title={$marketPlayer.isPlaying ? 'Pause' : 'Play'}
		>
			{#if $marketPlayer.isPlaying}
				<svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
					<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
				</svg>
			{:else}
				<svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
					<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
				</svg>
			{/if}
		</button>

		<!-- Next -->
		<button
			class="btn btn-sm variant-filled-surface"
			on:click={() => marketPlayer.next()}
			disabled={!$canGoForward}
			title="Next message"
		>
			<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
				<path d="M4.555 5.168A1 1 0 003 6v8a1 1 0 001.555.832L10 11.202V14a1 1 0 001.555.832l6-4a1 1 0 000-1.664l-6-4A1 1 0 0010 6v2.798l-5.445-3.63z" />
			</svg>
		</button>

		<!-- Jump to End -->
		<button
			class="btn btn-sm variant-filled-surface"
			on:click={() => marketPlayer.jumpToEnd()}
			disabled={!$canGoForward}
			title="Jump to end"
		>
			<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
				<path d="M4.555 5.168A1 1 0 003 6v8a1 1 0 001.555.832l6-4V14a1 1 0 001.555.832l6-4a1 1 0 000-1.664l-6-4A1 1 0 0010 6v3.798l-6-4z" />
			</svg>
		</button>

		<!-- Speed Control -->
		<div class="flex items-center gap-2">
			<label for="speed" class="text-sm font-medium">Speed:</label>
			<select
				id="speed"
				bind:value={playbackSpeed}
				on:change={handleSpeedChange}
				class="select select-sm w-20"
			>
				{#each speeds as speed}
					<option value={speed}>{speed}x</option>
				{/each}
			</select>
		</div>
	</div>
</div>
