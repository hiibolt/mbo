<script lang="ts">
	import { currentMessage } from '$lib/stores/marketPlayer';
	import { formatPrice, getBookEffect, getBookEffectError, isAddEffect, isCancelEffect, isModifyEffect } from '$lib/types';
	import type { BookEffect } from '$lib/types';

	$: mboMsg = $currentMessage?.mbo_msg;
	$: marketEffect = $currentMessage?.market_effect;
	$: bookEffect = marketEffect ? getBookEffect(marketEffect) : null;
	$: bookError = marketEffect ? getBookEffectError(marketEffect) : null;

	function getActionColor(action: string): string {
		switch (action.toString().toLowerCase()) {
			case 'add': return 'text-green-600';
			case 'cancel': return 'text-red-600';
			case 'modify': return 'text-yellow-600';
			case 'trade':
			case 'fill': return 'text-blue-600';
			default: return 'text-surface-600-300-token';
		}
	}

	function getSideColor(side: string): string {
		switch (side.toString().toLowerCase()) {
			case 'bid': return 'text-green-600';
			case 'ask': return 'text-red-600';
			default: return 'text-surface-600-300-token';
		}
	}

	function formatBookEffect(effect: BookEffect): string {
		if (isAddEffect(effect)) {
			return `Add ${effect.Add.side} @ ${formatPrice(effect.Add.price)} × ${effect.Add.size}`;
		} else if (isCancelEffect(effect)) {
			return `Cancel ${effect.Cancel.side} @ ${formatPrice(effect.Cancel.price)} × ${effect.Cancel.size}`;
		} else if (isModifyEffect(effect)) {
			return `Modify ${effect.Modify.side}: ${formatPrice(effect.Modify.old_price)} → ${formatPrice(effect.Modify.new_price)}, size ${effect.Modify.old_size} → ${effect.Modify.new_size}`;
		}
		return 'Unknown';
	}
</script>

<div class="card p-4 space-y-4">
	<h3 class="h4 font-bold">Current Message & Effect</h3>

	{#if !$currentMessage}
		<div class="text-center text-surface-600-300-token py-8">
			<p>No message selected</p>
			<p class="text-sm">Use the player controls to navigate</p>
		</div>
	{:else}
		<!-- MBO Message -->
		<div class="space-y-2">
			<h4 class="font-semibold text-primary-500">MBO Message</h4>
			<div class="grid grid-cols-2 gap-2 text-sm">
				<div>
					<span class="text-surface-600-300-token">Action:</span>
					<span class={`font-semibold ml-2 ${getActionColor(mboMsg.action)}`}>
						{mboMsg.action}
					</span>
				</div>
				<div>
					<span class="text-surface-600-300-token">Side:</span>
					<span class={`font-semibold ml-2 ${getSideColor(mboMsg.side)}`}>
						{mboMsg.side}
					</span>
				</div>
				<div>
					<span class="text-surface-600-300-token">Price:</span>
					<span class="font-mono ml-2">${formatPrice(mboMsg.price)}</span>
				</div>
				<div>
					<span class="text-surface-600-300-token">Size:</span>
					<span class="font-mono ml-2">{mboMsg.size}</span>
				</div>
				<div class="col-span-2">
					<span class="text-surface-600-300-token">Order ID:</span>
					<span class="font-mono ml-2">{mboMsg.order_id}</span>
				</div>
				<div>
					<span class="text-surface-600-300-token">Sequence:</span>
					<span class="font-mono ml-2">{mboMsg.sequence}</span>
				</div>
				<div>
					<span class="text-surface-600-300-token">Channel:</span>
					<span class="font-mono ml-2">{mboMsg.channel_id}</span>
				</div>
			</div>
		</div>

		<hr class="!border-t-2" />

		<!-- Market Effect -->
		<div class="space-y-2">
			<h4 class="font-semibold text-primary-500">Market Effect</h4>
			
			{#if marketEffect?.publisher_created}
				<div class="alert variant-filled-success">
					<span>🆕 New Publisher Created: {marketEffect.publisher_created}</span>
				</div>
			{/if}

			{#if bookError}
				<div class="alert variant-filled-error">
					<span>❌ Error: {bookError}</span>
				</div>
			{:else if bookEffect}
				<div class="alert variant-filled-success">
					<span>✅ {formatBookEffect(bookEffect)}</span>
				</div>
			{:else}
				<div class="alert variant-soft-surface">
					<span>ℹ️ No book effect (Trade/Fill/etc)</span>
				</div>
			{/if}
		</div>
	{/if}
</div>
