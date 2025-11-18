import { writable, derived, get } from 'svelte/store';
import { type Market, type MBOMsgEffect, type MarketEffect, type BookEffect, type Book, type MboMsg, type Publisher, publisherIdToString, type PublisherStr } from '$lib/types';
import type { StreamStatus } from '$lib/services/streamClient';

export interface PlayerState {
  messages: MBOMsgEffect[];
  currentIndex: number;
  market: Market;
  isPlaying: boolean;
  isLoading: boolean;
  loadProgress: number;
  streamStatus: StreamStatus;
  playbackSpeed: number;
  error?: string;
}

const initialState: PlayerState = {
  messages: [],
  currentIndex: -1,
  market: { books: {} },
  isPlaying: false,
  isLoading: false,
  loadProgress: 0,
  streamStatus: 'idle',
  playbackSpeed: 1,
};

// Create the main store
function createMarketPlayer() {
  const { subscribe, set, update } = writable<PlayerState>(initialState);
  
  let playbackInterval: ReturnType<typeof setInterval> | null = null;

  return {
    subscribe,
    
    // Loading
    setLoading: (isLoading: boolean, status: StreamStatus) => 
      update(s => ({ ...s, isLoading, streamStatus: status })),
    
    setLoadProgress: (loaded: number, total?: number) => 
      update(s => ({ 
        ...s, 
        loadProgress: total ? (loaded / total) * 100 : 0 
      })),
    
    setMessages: (messages: MBOMsgEffect[]) => 
      update(s => ({ ...s, messages, isLoading: false, streamStatus: 'complete' })),
    
    setError: (error: string) => 
      update(s => ({ ...s, error, isLoading: false, streamStatus: 'error' })),
    
    // Playback control
    play: () => {
      const state = get({ subscribe });
      if (playbackInterval) return; // Already playing
      
      update(s => ({ ...s, isPlaying: true }));
      
      const baseInterval = 50; // 50ms per message
      playbackInterval = setInterval(() => {
        const currentState = get({ subscribe });
        
        if (currentState.currentIndex >= currentState.messages.length - 1) {
          // Reached the end
          if (playbackInterval) {
            clearInterval(playbackInterval);
            playbackInterval = null;
          }
          update(s => ({ ...s, isPlaying: false }));
          return;
        }
        
        // Move to next message
        update(s => {
          const newIndex = s.currentIndex + 1;
          const newMarket = applyEffect(s.market, s.messages[newIndex]);
          return { ...s, currentIndex: newIndex, market: newMarket };
        });
      }, baseInterval / state.playbackSpeed);
    },
    
    pause: () => {
      if (playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
      }
      update(s => ({ ...s, isPlaying: false }));
    },
    
    // Navigation
    next: () => {
      update(s => {
        if (s.currentIndex >= s.messages.length - 1) return s;
        const newIndex = s.currentIndex + 1;
        const newMarket = applyEffect(s.market, s.messages[newIndex]);
        return { ...s, currentIndex: newIndex, market: newMarket, isPlaying: false };
      });
      
      if (playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
      }
    },
    
    previous: () => {
      update(s => {
        if (s.currentIndex < 0) return s;
        // Unapply the current effect to go back
        const newMarket = unapplyEffect(s.market, s.messages[s.currentIndex]);
        const newIndex = s.currentIndex - 1;
        return { ...s, currentIndex: newIndex, market: newMarket, isPlaying: false };
      });
      
      if (playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
      }
    },
    
    jumpToStart: () => {
      if (playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
      }
      update(s => ({ 
        ...s, 
        currentIndex: -1, 
        market: { books: {} }, 
        isPlaying: false 
      }));
    },
    
    jumpToEnd: () => {
      if (playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
      }
      update(s => {
        const newMarket = rebuildMarketToIndex(s.messages, s.messages.length - 1);
        return { 
          ...s, 
          currentIndex: s.messages.length - 1, 
          market: newMarket, 
          isPlaying: false 
        };
      });
    },
    
    jumpToIndex: (index: number) => {
      if (playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
      }
      update(s => {
        if (index < -1 || index >= s.messages.length) return s;
        
        // Rebuild from scratch when jumping
        const newMarket = index === -1 ? { books: {} } : rebuildMarketToIndex(s.messages, index);
        return { ...s, currentIndex: index, market: newMarket, isPlaying: false };
      });
    },
    
    setPlaybackSpeed: (speed: number) => {
      const wasPlaying = get({ subscribe }).isPlaying;
      if (wasPlaying && playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
        update(s => ({ ...s, playbackSpeed: speed, isPlaying: false }));
      } else {
        update(s => ({ ...s, playbackSpeed: speed }));
      }
    },
    
    reset: () => {
      if (playbackInterval) {
        clearInterval(playbackInterval);
        playbackInterval = null;
      }
      set(initialState);
    },
  };
}

/**
 * Apply an effect to the market (going forward)
 */
function applyEffect(market: Market, msgEffect: MBOMsgEffect): Market {
  const newMarket = JSON.parse(JSON.stringify(market)) as Market; // Deep clone
  
  const mboMsg = msgEffect.mbo_msg;
  const effect = msgEffect.market_effect;
  const instrumentId = mboMsg.hd.instrument_id;
  const publisherId = publisherIdToString(mboMsg.hd.publisher_id);
  
  // Ensure instrument exists
  if (!newMarket.books[instrumentId]) {
    newMarket.books[instrumentId] = [];
  }
  
  // Handle publisher creation
  if (effect.publisher_created) {
    newMarket.books[instrumentId].push([
      effect.publisher_created, 
      { orders_by_id: {}, offers: {}, bids: {} }
    ]);
  }
  
  // Find the book for this publisher
  const publisherBooks: [PublisherStr, Book][] = newMarket.books[instrumentId];
  let bookIndex = publisherBooks.findIndex(([pub, _]) => {
    console.log(pub, publisherId);
    return pub === publisherId;
  });
  
  if (bookIndex === -1) {
    console.log(`Warning - publisher ${publisherId} doesn't yet exist?`);
    console.dir(publisherBooks);
    return newMarket; // Publisher doesn't exist yet
  }
  
  const [pub, book] = publisherBooks[bookIndex];
  
  // Apply book effect
  if ('Ok' in effect.book_effect && effect.book_effect.Ok) {
    applyBookEffect(book, effect.book_effect.Ok, mboMsg);
  }
  
  publisherBooks[bookIndex] = [pub, book];
  
  return newMarket;
}

/**
 * Unapply an effect from the market (going backward)
 */
function unapplyEffect(market: Market, msgEffect: MBOMsgEffect): Market {
  const newMarket = JSON.parse(JSON.stringify(market)) as Market;
  
  const mboMsg = msgEffect.mbo_msg;
  const effect = msgEffect.market_effect;
  const instrumentId = mboMsg.hd.instrument_id;
  const publisherId: PublisherStr = publisherIdToString(mboMsg.hd.publisher_id);
  
  if (!newMarket.books[instrumentId]) {
    return newMarket;
  }
  
  // Handle publisher removal (undo creation)
  if (effect.publisher_created) {
    const publisherBooks: [PublisherStr, Book][] = newMarket.books[instrumentId];
    const idx = publisherBooks.findIndex(([pub, _]) => {
      console.log(pub, publisherId);
      return pub === publisherId;
    });
    if (idx !== -1) {
      publisherBooks.splice(idx, 1);
    }
    if (publisherBooks.length === 0) {
      delete newMarket.books[instrumentId];
    }
    return newMarket;
  }
  
  // Find the book
  const publisherBooks = newMarket.books[instrumentId];
  let bookIndex = publisherBooks.findIndex(([pub, _]) => pub === publisherId);
  
  if (bookIndex === -1) {
    return newMarket;
  }
  
  const [pub, book] = publisherBooks[bookIndex];
  
  // Unapply book effect (do the opposite)
  if ('Ok' in effect.book_effect && effect.book_effect.Ok) {
    unapplyBookEffect(book, effect.book_effect.Ok, mboMsg);
  }
  
  publisherBooks[bookIndex] = [pub, book];
  
  return newMarket;
}

/**
 * Apply a BookEffect to a Book
 */
function applyBookEffect(book: Book, effect: BookEffect, mboMsg: MboMsg) {
  if ('Add' in effect) {
    const { side, price } = effect.Add;
    const levels = side === 'Bid' ? book.bids : book.offers;
    
    if (!levels[price]) {
      levels[price] = [];
    }
    
    levels[price].push(mboMsg);
    book.orders_by_id[mboMsg.order_id] = [side, price];
    
  } else if ('Cancel' in effect) {
    const { side, price, size } = effect.Cancel;
    const levels = side === 'Bid' ? book.bids : book.offers;
    
    if (levels[price]) {
      const priceLevel = levels[price];
      const orderIndex = priceLevel.findIndex(o => o.order_id === mboMsg.order_id);
      
      if (orderIndex !== -1) {
        priceLevel[orderIndex].size -= size;
        
        if (priceLevel[orderIndex].size <= 0) {
          priceLevel.splice(orderIndex, 1);
          delete book.orders_by_id[mboMsg.order_id];
          
          if (priceLevel.length === 0) {
            delete levels[price];
          }
        }
      }
    }
    
  } else if ('Modify' in effect) {
    const { side, old_price, new_price, new_size } = effect.Modify;
    const levels = side === 'Bid' ? book.bids : book.offers;
    
    // Remove from old price
    if (levels[old_price]) {
      const idx = levels[old_price].findIndex(o => o.order_id === mboMsg.order_id);
      if (idx !== -1) {
        levels[old_price].splice(idx, 1);
        if (levels[old_price].length === 0) {
          delete levels[old_price];
        }
      }
    }
    
    // Add to new price
    if (!levels[new_price]) {
      levels[new_price] = [];
    }
    const modifiedMsg = { ...mboMsg, price: new_price, size: new_size };
    levels[new_price].push(modifiedMsg);
    book.orders_by_id[mboMsg.order_id] = [side, new_price];
  }
}

/**
 * Unapply a BookEffect (reverse the operation)
 */
function unapplyBookEffect(book: Book, effect: BookEffect, mboMsg: MboMsg) {
  if ('Add' in effect) {
    // Undo Add = Remove the order
    const { side, price } = effect.Add;
    const levels = side === 'Bid' ? book.bids : book.offers;
    
    if (levels[price]) {
      const idx = levels[price].findIndex(o => o.order_id === mboMsg.order_id);
      if (idx !== -1) {
        levels[price].splice(idx, 1);
        if (levels[price].length === 0) {
          delete levels[price];
        }
      }
    }
    delete book.orders_by_id[mboMsg.order_id];
    
  } else if ('Cancel' in effect) {
    // Undo Cancel = Add the order back
    const { side, price, size } = effect.Cancel;
    const levels = side === 'Bid' ? book.bids : book.offers;
    
    if (!levels[price]) {
      levels[price] = [];
    }
    
    // Check if order exists and just increase size, or add it
    const existingIdx = levels[price].findIndex(o => o.order_id === mboMsg.order_id);
    if (existingIdx !== -1) {
      levels[price][existingIdx].size += size;
    } else {
      levels[price].push({ ...mboMsg, size });
      book.orders_by_id[mboMsg.order_id] = [side, price];
    }
    
  } else if ('Modify' in effect) {
    // Undo Modify = Move back to old price with old size
    const { side, old_price, new_price, old_size } = effect.Modify;
    const levels = side === 'Bid' ? book.bids : book.offers;
    
    // Remove from new price
    if (levels[new_price]) {
      const idx = levels[new_price].findIndex(o => o.order_id === mboMsg.order_id);
      if (idx !== -1) {
        levels[new_price].splice(idx, 1);
        if (levels[new_price].length === 0) {
          delete levels[new_price];
        }
      }
    }
    
    // Add back to old price with old size
    if (!levels[old_price]) {
      levels[old_price] = [];
    }
    const originalMsg = { ...mboMsg, price: old_price, size: old_size };
    levels[old_price].push(originalMsg);
    book.orders_by_id[mboMsg.order_id] = [side, old_price];
  }
}

/**
 * Rebuild market from scratch to a specific index (used for jumps)
 */
function rebuildMarketToIndex(messages: MBOMsgEffect[], toIndex: number): Market {
  let market: Market = { books: {} };
  
  for (let i = 0; i <= toIndex; i++) {
    market = applyEffect(market, messages[i]);
  }
  
  return market;
}

export const marketPlayer = createMarketPlayer();

// Derived stores for convenience
export const currentMessage = derived(
  marketPlayer,
  $player => $player.currentIndex >= 0 ? $player.messages[$player.currentIndex] : null
);

export const progress = derived(
  marketPlayer,
  $player => ({
    current: $player.currentIndex + 1,
    total: $player.messages.length,
    percentage: $player.messages.length > 0 
      ? ((($player.currentIndex + 1) / $player.messages.length) * 100)
      : 0
  })
);

export const canPlayPause = derived(
  marketPlayer,
  $player => $player.messages.length > 0 && $player.currentIndex < $player.messages.length - 1
);

export const canGoBack = derived(
  marketPlayer,
  $player => $player.currentIndex > -1
);

export const canGoForward = derived(
  marketPlayer,
  $player => $player.messages.length > 0 && $player.currentIndex < $player.messages.length - 1
);
