<script lang="ts">
	import { ProgressRadial } from '@skeletonlabs/skeleton';
</script>

<div class="p-12 flex flex-col items-center justify-center space-y-4">
	<ProgressRadial width="w-32" stroke={60} meter="stroke-blue-600" track="stroke-gray-200" />
	<h2 class="text-2xl font-bold text-black">Loading Market Snapshot...</h2>
	<p class="text-sm text-gray-600">Fetching order book data from backend</p>
</div>
