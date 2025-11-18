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

/**f
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
export type Publisher = number;
export type PublisherStr = string;

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
  publisher_created: PublisherStr | null;
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
  books: Record<number, Array<[PublisherStr, Book]>>;
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
 * Converts a publisher ID to its string representation
 */
export function publisherIdToString(publisherId: number): string {
  const publisherMap: Record<number, string> = {
    1: "GLBX.MDP3.GLBX",
    2: "XNAS.ITCH.XNAS",
    3: "XBOS.ITCH.XBOS",
    4: "XPSX.ITCH.XPSX",
    5: "BATS.PITCH.BATS",
    6: "BATY.PITCH.BATY",
    7: "EDGA.PITCH.EDGA",
    8: "EDGX.PITCH.EDGX",
    9: "XNYS.PILLAR.XNYS",
    10: "XCIS.PILLAR.XCIS",
    11: "XASE.PILLAR.XASE",
    12: "XCHI.PILLAR.XCHI",
    13: "XCIS.BBO.XCIS",
    14: "XCIS.TRADES.XCIS",
    15: "MEMX.MEMOIR.MEMX",
    16: "EPRL.DOM.EPRL",
    17: "XNAS.NLS.FINN",
    18: "XNAS.NLS.FINC",
    19: "XNYS.TRADES.FINY",
    20: "OPRA.PILLAR.AMXO",
    21: "OPRA.PILLAR.XBOX",
    22: "OPRA.PILLAR.XCBO",
    23: "OPRA.PILLAR.EMLD",
    24: "OPRA.PILLAR.EDGO",
    25: "OPRA.PILLAR.GMNI",
    26: "OPRA.PILLAR.XISX",
    27: "OPRA.PILLAR.MCRY",
    28: "OPRA.PILLAR.XMIO",
    29: "OPRA.PILLAR.ARCO",
    30: "OPRA.PILLAR.OPRA",
    31: "OPRA.PILLAR.MPRL",
    32: "OPRA.PILLAR.XNDQ",
    33: "OPRA.PILLAR.XBXO",
    34: "OPRA.PILLAR.C2OX",
    35: "OPRA.PILLAR.XPHL",
    36: "OPRA.PILLAR.BATO",
    37: "OPRA.PILLAR.MXOP",
    38: "IEXG.TOPS.IEXG",
    39: "DBEQ.BASIC.XCHI",
    40: "DBEQ.BASIC.XCIS",
    41: "DBEQ.BASIC.IEXG",
    42: "DBEQ.BASIC.EPRL",
    43: "ARCX.PILLAR.ARCX",
    44: "XNYS.BBO.XNYS",
    45: "XNYS.TRADES.XNYS",
    46: "XNAS.QBBO.XNAS",
    47: "XNAS.NLS.XNAS",
    48: "EQUS.PLUS.XCHI",
    49: "EQUS.PLUS.XCIS",
    50: "EQUS.PLUS.IEXG",
    51: "EQUS.PLUS.EPRL",
    52: "EQUS.PLUS.XNAS",
    53: "EQUS.PLUS.XNYS",
    54: "EQUS.PLUS.FINN",
    55: "EQUS.PLUS.FINY",
    56: "EQUS.PLUS.FINC",
    57: "IFEU.IMPACT.IFEU",
    58: "NDEX.IMPACT.NDEX",
    59: "DBEQ.BASIC.DBEQ",
    60: "EQUS.PLUS.EQUS",
    61: "OPRA.PILLAR.SPHR",
    62: "EQUS.ALL.XCHI",
    63: "EQUS.ALL.XCIS",
    64: "EQUS.ALL.IEXG",
    65: "EQUS.ALL.EPRL",
    66: "EQUS.ALL.XNAS",
    67: "EQUS.ALL.XNYS",
    68: "EQUS.ALL.FINN",
    69: "EQUS.ALL.FINY",
    70: "EQUS.ALL.FINC",
    71: "EQUS.ALL.BATS",
    72: "EQUS.ALL.BATY",
    73: "EQUS.ALL.EDGA",
    74: "EQUS.ALL.EDGX",
    75: "EQUS.ALL.XBOS",
    76: "EQUS.ALL.XPSX",
    77: "EQUS.ALL.MEMX",
    78: "EQUS.ALL.XASE",
    79: "EQUS.ALL.ARCX",
    80: "EQUS.ALL.LTSE",
    81: "XNAS.BASIC.XNAS",
    82: "XNAS.BASIC.FINN",
    83: "XNAS.BASIC.FINC",
    84: "IFEU.IMPACT.XOFF",
    85: "NDEX.IMPACT.XOFF",
    86: "XNAS.NLS.XBOS",
    87: "XNAS.NLS.XPSX",
    88: "XNAS.BASIC.XBOS",
    89: "XNAS.BASIC.XPSX",
    90: "EQUS.SUMMARY.EQUS",
    91: "XCIS.TRADESBBO.XCIS",
    92: "XNYS.TRADESBBO.XNYS",
    93: "XNAS.BASIC.EQUS",
    94: "EQUS.ALL.EQUS",
    95: "EQUS.MINI.EQUS",
    96: "XNYS.TRADES.EQUS",
    97: "IFUS.IMPACT.IFUS",
    98: "IFUS.IMPACT.XOFF",
    99: "IFLL.IMPACT.IFLL",
    100: "IFLL.IMPACT.XOFF",
    101: "XEUR.EOBI.XEUR",
    102: "XEEE.EOBI.XEEE",
    103: "XEUR.EOBI.XOFF",
    104: "XEEE.EOBI.XOFF",
  };

  return publisherMap[publisherId] || `Unknown Publisher (${publisherId})`;
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
