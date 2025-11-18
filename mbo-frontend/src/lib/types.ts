// TypeScript types matching the Rust backend API
// Generated from mbo-backend/src/datatypes/book.rs and market.rs

/**
 * Represents different order sides in the market
 */
export type Side = 'None' | 'Ask' | 'Bid';

/**
 * Represents different actions that can be performed on orders
 */
export type Action = 'Add' | 'Cancel' | 'Modify' | 'Clear' | 'Trade' | 'Fill' | 'None';

/**
 * Effects that operations have on the order book
 */
export type BookEffect = 
  | { Add: { side: Side; price: number; size: number } }
  | { Cancel: { side: Side; price: number; size: number } }
  | { Modify: { side: Side; old_price: number; new_price: number; old_size: number; new_size: number } };

/**
 * A price level in the order book
 */
export interface PriceLevel {
  price: number;
  size: number;
  count: number;
}

/**
 * Publisher information (venue/exchange)
 */
export interface Publisher {
  publisher_id: number;
  dataset: string;
  venue: string;
}

/**
 * Order book header
 */
export interface RecordHeader {
  length: number;
  rtype: number;
  publisher_id: number;
  instrument_id: number;
  ts_event: number;
}

/**
 * Market-By-Order message from Databento
 */
export interface MboMsg {
  hd: RecordHeader;
  order_id: number;
  price: number;
  size: number;
  flags: number;
  channel_id: number;
  action: string;
  side: string;
  ts_recv: number;
  ts_in_delta: number;
  sequence: number;
}

/**
 * Market effect resulting from applying an MBO message
 */
export interface MarketEffect {
  publisher_created: Publisher | null;
  book_effect: { Ok: BookEffect | null } | { Err: string };
}

/**
 * Combined MBO message and its effect on the market
 */
export interface MBOMsgEffect {
  mbo_msg: MboMsg;
  market_effect: MarketEffect;
}

/**
 * Order book structure with bids and offers organized by price level
 */
export interface Book {
  orders_by_id: Record<number, [Side, number]>;
  offers: Record<number, MboMsg[]>;
  bids: Record<number, MboMsg[]>;
}

/**
 * Market state containing all books organized by instrument and publisher
 */
export interface Market {
  books: Record<number, Array<[Publisher, Book]>>;
}

/**
 * Complete market snapshot including the market state and the effect that created it
 */
export interface MarketSnapshot {
  market: Market;
  mbomsg_effect: MBOMsgEffect;
}

/**
 * Aggregated best bid and offer across all publishers
 */
export interface AggregatedBBO {
  best_bid: PriceLevel | null;
  best_offer: PriceLevel | null;
}

/**
 * Order book display data for UI
 */
export interface OrderBook {
  symbol: string;
  timestamp: string;
  best_bid: PriceLevel | null;
  best_offer: PriceLevel | null;
}

/**
 * Error state for UI components
 */
export interface ErrorState {
  hasError: boolean;
  message: string;
  details?: string;
  canRetry: boolean;
}

/**
 * Helper function to check if a BookEffect is an Add
 */
export function isAddEffect(effect: BookEffect): effect is { Add: { side: Side; price: number; size: number } } {
  return 'Add' in effect;
}

/**
 * Helper function to check if a BookEffect is a Cancel
 */
export function isCancelEffect(effect: BookEffect): effect is { Cancel: { side: Side; price: number; size: number } } {
  return 'Cancel' in effect;
}

/**
 * Helper function to check if a BookEffect is a Modify
 */
export function isModifyEffect(effect: BookEffect): effect is { Modify: { side: Side; old_price: number; new_price: number; old_size: number; new_size: number } } {
  return 'Modify' in effect;
}

/**
 * Helper function to extract book effect from Result type
 */
export function getBookEffect(marketEffect: MarketEffect): BookEffect | null {
  if ('Ok' in marketEffect.book_effect) {
    return marketEffect.book_effect.Ok;
  }
  return null;
}

/**
 * Helper function to get error message from Result type
 */
export function getBookEffectError(marketEffect: MarketEffect): string | null {
  if ('Err' in marketEffect.book_effect) {
    return marketEffect.book_effect.Err;
  }
  return null;
}

/**
 * Format price from i64 (price in nanodollars) to dollar string
 */
export function formatPrice(price: number | null): string {
  if (price === null) return 'N/A';
  return (price / 1e9).toFixed(2);
}

/**
 * Format size for display
 */
export function formatSize(size: number | null): string {
  if (size === null) return 'N/A';
  return size.toLocaleString();
}

/**
 * Calculate spread between bid and ask
 */
export function calculateSpread(bid: PriceLevel | null, ask: PriceLevel | null): number | null {
  if (!bid || !ask) return null;
  return ask.price - bid.price;
}

/**
 * Calculate mid price between bid and ask
 */
export function calculateMidPrice(bid: PriceLevel | null, ask: PriceLevel | null): number | null {
  if (!bid || !ask) return null;
  return (bid.price + ask.price) / 2;
}
