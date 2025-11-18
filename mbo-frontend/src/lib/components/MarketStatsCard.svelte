<script lang="ts">
	export let spread: string;
	export let midPrice: string;

	interface Stat {
		label: string;
		value: string;
	}

	$: stats = [
		{ label: 'SPREAD', value: `$${spread}` },
		{ label: 'MID PRICE', value: `$${midPrice}` },
		{ label: 'TYPE', value: 'Snapshot' }
	] as Stat[];
</script>

<div class="p-6">
	<header class="mb-4">
		<h3 class="text-2xl font-bold text-black">Market Stats</h3>
	</header>
	<section class="space-y-4">
		{#each stats as stat}
			<div class="flex justify-between items-center bg-gray-50 rounded p-4">
				<span class="text-xs text-gray-500 font-semibold">{stat.label}</span>
				<span class="text-2xl font-bold text-black">{stat.value}</span>
			</div>
		{/each}
	</section>
</div>
