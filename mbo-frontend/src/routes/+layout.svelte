<script lang="ts">
	import '../app.postcss';
	import { AppShell, AppBar, Toast } from '@skeletonlabs/skeleton';
	import { initializeStores } from '@skeletonlabs/skeleton';
	import { onMount } from 'svelte';

	// Initialize Skeleton stores
	initializeStores();

	// Error boundary state
	let hasError = false;
	let errorMessage = '';

	// Global error handler
	onMount(() => {
		window.addEventListener('error', (event) => {
			hasError = true;
			errorMessage = event.message || 'An unknown error occurred';
			console.error('Global error:', event.error);
		});

		window.addEventListener('unhandledrejection', (event) => {
			hasError = true;
			errorMessage = event.reason?.message || 'An unhandled promise rejection occurred';
			console.error('Unhandled rejection:', event.reason);
		});
	});
</script>

<Toast />

<AppShell>
	<svelte:fragment slot="header">
		<AppBar>
			<svelte:fragment slot="lead">
				<strong class="text-xl uppercase"></strong>
			</svelte:fragment>
			<svelte:fragment slot="trail">
				<div class="flex items-center gap-4">
					<span class="badge variant-filled-success">Live</span>
				</div>
			</svelte:fragment>
		</AppBar>
	</svelte:fragment>

	{#if hasError}
		<div class="container h-full mx-auto flex justify-center items-center">
			<div class="card p-8 variant-filled-error max-w-2xl">
				<header class="card-header">
					<h2 class="h2">Application Error</h2>
				</header>
				<section class="p-4">
					<p class="text-lg mb-4">{errorMessage}</p>
					<button
						class="btn variant-filled"
						on:click={() => {
							hasError = false;
							errorMessage = '';
							window.location.reload();
						}}
					>
						Reload Application
					</button>
				</section>
			</div>
		</div>
	{:else}
		<slot />
	{/if}
</AppShell>
