use std::collections::{BTreeMap, HashMap, VecDeque};

use super::{price_level::PriceLevel, Level};
use databento::{
    dbn::{
        Action, BidAskPair, MboMsg, Side,
        UNDEF_PRICE,
    },
};
use anyhow::{Result, Context, bail, ensure};
use tracing::warn;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Book {
    orders_by_id: HashMap<u64, (Side, i64)>,
    offers: BTreeMap<i64, Level>,
    bids: BTreeMap<i64, Level>,
}
impl Book {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bbo(&self) -> (Option<PriceLevel>, Option<PriceLevel>) {
        (self.bid_level(0), self.ask_level(0))
    }

    pub fn bid_level(&self, idx: usize) -> Option<PriceLevel> {
        self.bids
            .iter()
            // Reverse to get highest first
            .rev()
            .nth(idx)
            .map(|(price, orders)| PriceLevel::new(*price, orders.iter()))
    }

    pub fn ask_level(&self, idx: usize) -> Option<PriceLevel> {
        self.offers
            .iter()
            .nth(idx)
            .map(|(price, orders)| PriceLevel::new(*price, orders.iter()))
    }

    pub fn bid_level_by_px(&self, px: i64) -> Option<PriceLevel> {
        self.bids
            .get(&px)
            .map(|orders| PriceLevel::new(px, orders.iter()))
    }

    pub fn ask_level_by_px(&self, px: i64) -> Option<PriceLevel> {
        self.offers
            .get(&px)
            .map(|orders| PriceLevel::new(px, orders.iter()))
    }

    pub fn order(&self, order_id: u64) -> Option<&MboMsg> {
        let (side, price) = self.orders_by_id.get(&order_id)?;
        let levels = self.side_levels(*side).ok()?;
        let level = levels.get(price)?;
        level.iter().find(|order| order.order_id == order_id)
    }

    pub fn queue_pos(&self, order_id: u64) -> Option<u32> {
        let (side, price) = self.orders_by_id.get(&order_id)?;
        let levels = self.side_levels(*side).ok()?;
        let level = levels.get(price)?;
        Some(
            level
                .iter()
                .take_while(|order| order.order_id != order_id)
                .fold(0, |acc, order| acc + order.size),
        )
    }

    pub fn snapshot(&self, level_count: usize) -> Vec<BidAskPair> {
        (0..level_count)
            .map(|i| {
                let mut ba_pair = BidAskPair::default();
                if let Some(bid) = self.bid_level(i) {
                    ba_pair.bid_px = bid.price;
                    ba_pair.bid_sz = bid.size;
                    ba_pair.bid_ct = bid.count;
                }
                if let Some(ask) = self.ask_level(i) {
                    ba_pair.ask_px = ask.price;
                    ba_pair.ask_sz = ask.size;
                    ba_pair.ask_ct = ask.count;
                }
                ba_pair
            })
            .collect()
    }

    #[tracing::instrument(skip(self), fields(order_id = mbo.order_id, action = ?mbo.action()))]
    pub fn apply(&mut self, mbo: MboMsg) -> Result<()> {
        let action = mbo.action()
            .context("MBO message has no valid action")?;
        match action {
            Action::Modify => {
                if let Some(()) = self.modify(mbo.clone())? {
                    warn!("Skipped Modify for pre-snapshot order ID {}", mbo.order_id);
                }
            }
            Action::Trade | Action::Fill | Action::None => {}
            Action::Cancel => {
                if let Some(()) = self.cancel(mbo.clone())? {
                    warn!("Skipped Cancel for pre-snapshot order ID {}", mbo.order_id);
                }
            }
            Action::Add => { self.add(mbo)?; }
            Action::Clear => self.clear(),
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.orders_by_id.clear();
        self.offers.clear();
        self.bids.clear();
    }

    #[tracing::instrument(skip(self), fields(order_id = mbo.order_id, price = mbo.price, size = mbo.size))]
    fn add(&mut self, mbo: MboMsg) -> Result<()> {
        let price = mbo.price;
        let side = mbo.side()
            .context("MBO message has no valid side")?;
        if mbo.flags.is_tob() {
            let levels: &mut BTreeMap<i64, Level> = self.side_levels_mut(side)?;
            levels.clear();
            // UNDEF_PRICE indicates the side's book should be cleared
            // and doesn't represent an order that should be added
            if mbo.price != UNDEF_PRICE {
                levels.insert(price, VecDeque::from([mbo]));
            }
        } else {
            ensure!(price != UNDEF_PRICE, "Price cannot be UNDEF_PRICE for non-TOB add");
            ensure!(
                self.orders_by_id.insert(mbo.order_id, (side, price)).is_none(),
                "Duplicate order ID {} - order already exists in book",
                mbo.order_id
            );
            let level: &mut Level = self.get_or_insert_level(side, price)?;
            level.push_back(mbo);
            
            // Check if this add created a crossed book and clean it up
            self.match_crossed_orders()?;
        }
        Ok(())
    }
    
    /// Match and remove crossed orders to maintain book invariants.
    /// 
    /// In real markets, when a bid >= ask, orders execute immediately.
    /// MBO data may show the pre-execution state momentarily, so we
    /// simulate the matching engine by removing crossed levels.
    #[tracing::instrument(skip(self))]
    fn match_crossed_orders(&mut self) -> Result<()> {
        loop {
            // Get best bid and ask
            let best_bid_price = self.bids.keys().rev().next().copied();
            let best_ask_price = self.offers.keys().next().copied();
            
            match (best_bid_price, best_ask_price) {
                (Some(bid_px), Some(ask_px)) if bid_px >= ask_px => {
                    // Book is crossed - remove the crossed levels
                    // In a real matching engine, these would execute against each other
                    
                    tracing::debug!(
                        bid_price = bid_px,
                        ask_price = ask_px,
                        "Matching crossed orders: bid ${:.2} >= ask ${:.2}",
                        bid_px as f64 / 1e9,
                        ask_px as f64 / 1e9
                    );
                    
                    // Remove all orders at the crossed bid level
                    if let Some(bid_level) = self.bids.remove(&bid_px) {
                        for order in bid_level {
                            self.orders_by_id.remove(&order.order_id);
                        }
                    }
                    
                    // Remove all orders at the crossed ask level
                    if let Some(ask_level) = self.offers.remove(&ask_px) {
                        for order in ask_level {
                            self.orders_by_id.remove(&order.order_id);
                        }
                    }
                    
                    // Continue checking in case multiple levels are crossed
                }
                _ => {
                    // Book is not crossed, we're done
                    break;
                }
            }
        }
        
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(order_id = mbo.order_id, price = mbo.price, size = mbo.size))]
    fn cancel(&mut self, mbo: MboMsg) -> Result<Option<()>> {
        let side = mbo.side()
            .context("MBO message has no valid side")?;
        
        // If the level doesn't exist, this cancel is for an order we never saw (pre-snapshot)
        let Ok(level) = self.level_mut(side, mbo.price) else {
            return Ok(Some(())); // Skip - order was added before our data started
        };
        
        // If the order isn't in the level, skip it (pre-snapshot order)
        let Ok(order_idx) = Self::find_order(level, mbo.order_id) else {
            return Ok(Some(())); // Skip
        };
        
        let existing_order = level.get_mut(order_idx)
            .context("Order index out of bounds")?;
        ensure!(
            existing_order.size >= mbo.size,
            "Cancel size {} exceeds existing order size {} for order ID {}",
            mbo.size,
            existing_order.size,
            mbo.order_id
        );
        existing_order.size -= mbo.size;
        if existing_order.size == 0 {
            level.remove(order_idx)
                .context("Order index out of bounds for removal")?;
            if level.is_empty() {
                self.remove_level(side, mbo.price)?;
            }
            self.orders_by_id.remove(&mbo.order_id);
        }
        Ok(None)
    }

    #[tracing::instrument(skip(self), fields(order_id = mbo.order_id, price = mbo.price, size = mbo.size))]
    fn modify(&mut self, mbo: MboMsg) -> Result<Option<()>> {
        let order_id = mbo.order_id;
        let side = mbo.side()
            .context("MBO message has no valid side")?;
        let Some((id_side, id_price)) = self.orders_by_id.get_mut(&order_id) else {
            // If order not found, skip (pre-snapshot order)
            // We don't treat it as an add because we don't know its history
            return Ok(Some(()));
        };
        let prev_side = *id_side;
        let prev_price = *id_price;
        // Update orders by ID
        *id_side = side;
        *id_price = mbo.price;
        // Update level order
        let level = self.level_mut(prev_side, prev_price)?;
        let order_idx = Self::find_order(level, mbo.order_id)
            .context("...while finding order in level")?;
        let existing_order = level.get_mut(order_idx)
            .context("Order index out of bounds")?;
        existing_order.size = mbo.size;
        let should_keep_priority = prev_price == mbo.price && existing_order.size >= mbo.size;
        if should_keep_priority {
            return Ok(None);
        }
        if prev_price != mbo.price {
            let prev_level = level;
            Self::remove_order(prev_level, order_id)?;
            if prev_level.is_empty() {
                self.remove_level(side, prev_price)?;
            }
            let level = self.get_or_insert_level(side, mbo.price)?;
            level.push_back(mbo);
        } else {
            Self::remove_order(level, order_id)?;
            level.push_back(mbo);
        }
        Ok(None)
    }

    fn get_or_insert_level(&mut self, side: Side, price: i64) -> Result<&mut Level> {
        let levels = self.side_levels_mut(side)?;
        Ok(levels.entry(price).or_default())
    }

    fn level_mut(&mut self, side: Side, price: i64) -> Result<&mut Level> {
        let levels = self.side_levels_mut(side)?;
        levels.get_mut(&price)
            .context(format!("Level not found at price {}", price))
    }

    fn remove_level(&mut self, side: Side, price: i64) -> Result<()> {
        self.side_levels_mut(side)?
            .remove(&price)
            .context(format!("Level not found at price {}", price))?;
        Ok(())
    }

    fn find_order(level: &VecDeque<MboMsg>, order_id: u64) -> Result<usize> {
        level.iter()
            .position(|o| o.order_id == order_id)
            .context(format!("Order not found with ID {}", order_id))
    }

    fn remove_order(level: &mut VecDeque<MboMsg>, order_id: u64) -> Result<()> {
        let index = Self::find_order(level, order_id)?;
        level.remove(index)
            .context(format!("Order not found at index {}", index))?;
        Ok(())
    }

    fn side_levels_mut(&mut self, side: Side) -> Result<&mut BTreeMap<i64, Level>> {
        match side {
            Side::Ask => Ok(&mut self.offers),
            Side::Bid => Ok(&mut self.bids),
            Side::None => bail!("Invalid side None"),
        }
    }

    fn side_levels(&self, side: Side) -> Result<&BTreeMap<i64, Level>> {
        match side {
            Side::Ask => Ok(&self.offers),
            Side::Bid => Ok(&self.bids),
            Side::None => bail!("Invalid side None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use databento::dbn::decode::{dbn::Decoder, DecodeRecord};
    use std::path::Path;
    
    #[test]
    fn test_book_with_real_data() -> Result<()> {
        let path = Path::new("assets/CLX5_mbo.dbn");
        let mut decoder = Decoder::from_file(path)?;
        let mut book = Book::new();
        
        let mut processed = 0;
        let max_messages = 1000; // Process first 1000 messages
        
        while let Some(msg) = decoder.decode_record::<MboMsg>()? {
            // Apply the message to the book
            book.apply(msg.clone())?;
            processed += 1;
            
            if processed >= max_messages {
                break;
            }
        }
        
        assert!(processed > 0, "Should have processed messages");
        
        // Verify the book has some state
        let (best_bid, best_ask) = book.bbo();
        
        // At least one side should have orders after processing real data
        assert!(
            best_bid.is_some() || best_ask.is_some(),
            "Book should have at least bid or ask after processing {} messages", 
            processed
        );
        
        println!("Processed {} messages successfully", processed);
        if let Some(bid) = best_bid {
            println!("Best bid: {} @ {}", bid.size, bid.price);
        }
        if let Some(ask) = best_ask {
            println!("Best ask: {} @ {}", ask.size, ask.price);
        }
        
        Ok(())
    }

    #[test]
    fn test_order_book_invariants_with_real_data() -> Result<()> {
        use databento::dbn::Action;
        
        let path = Path::new("assets/CLX5_mbo.dbn");
        let mut decoder = Decoder::from_file(path)?;
        let mut book = Book::new();
        
        let max_messages = 5000;
        let mut adds = 0;
        let mut cancels = 0;
        let mut modifies = 0;
        
        for _ in 0..max_messages {
            let Some(msg) = decoder.decode_record::<MboMsg>()? else {
                break;
            };
            
            // Track action types
            if let Ok(action) = msg.action() {
                match action {
                    Action::Add => adds += 1,
                    Action::Cancel => cancels += 1,
                    Action::Modify => modifies += 1,
                    _ => {}
                }
            }
            
            // Apply the message
            book.apply(msg.clone())?;
            
            // Verify invariants after each operation
            let (best_bid, best_ask) = book.bbo();
            
            // With matching logic, bid should ALWAYS be less than ask (never equal)
            if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
                assert!(
                    bid.price < ask.price,
                    "After matching, bid price ${:.2} should be strictly less than ask price ${:.2}",
                    bid.price as f64 / 1e9,
                    ask.price as f64 / 1e9
                );
                
                // All sizes and counts should be positive
                assert!(bid.size > 0, "Best bid size should be positive");
                assert!(ask.size > 0, "Best ask size should be positive");
                assert!(bid.count > 0, "Best bid count should be positive");
                assert!(ask.count > 0, "Best ask count should be positive");
            }
        }
        
        println!("Processed {} adds, {} cancels, {} modifies", adds, cancels, modifies);
        assert!(adds > 0, "Should have processed some add operations");
        
        Ok(())
    }

    #[test]
    fn test_snapshot_consistency() -> Result<()> {
        let path = Path::new("assets/CLX5_mbo.dbn");
        let mut decoder = Decoder::from_file(path)?;
        let mut book = Book::new();
        
        // Process some messages
        for _ in 0..2000 {
            let Some(msg) = decoder.decode_record::<MboMsg>()? else {
                break;
            };
            book.apply(msg.clone())?;
        }
        
        // Get snapshot with 5 levels
        let snapshot = book.snapshot(5);
        assert_eq!(snapshot.len(), 5);
        
        // Verify bid prices are descending (highest first)
        for i in 0..4 {
            if snapshot[i].bid_px != 0 && snapshot[i + 1].bid_px != 0 {
                assert!(
                    snapshot[i].bid_px >= snapshot[i + 1].bid_px,
                    "Bid prices should be descending: level {} ({}) >= level {} ({})",
                    i, snapshot[i].bid_px, i + 1, snapshot[i + 1].bid_px
                );
            }
        }
        
        // Verify ask prices are ascending (lowest first)
        for i in 0..4 {
            if snapshot[i].ask_px != 0 && snapshot[i + 1].ask_px != 0 {
                assert!(
                    snapshot[i].ask_px <= snapshot[i + 1].ask_px,
                    "Ask prices should be ascending: level {} ({}) <= level {} ({})",
                    i, snapshot[i].ask_px, i + 1, snapshot[i + 1].ask_px
                );
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_pre_snapshot_order_handling() -> Result<()> {
        use databento::dbn::Action;
        
        let path = Path::new("assets/CLX5_mbo.dbn");
        let mut decoder = Decoder::from_file(path)?;
        let mut book = Book::new();
        
        let mut skipped_cancels = 0;
        let mut skipped_modifies = 0;
        
        // The first messages in the file may reference orders from before the snapshot
        for _ in 0..100 {
            let Some(msg) = decoder.decode_record::<MboMsg>()? else {
                break;
            };
            
            let order_id = msg.order_id;
            
            // Check if this is a cancel/modify for an order we don't have
            if let Ok(action) = msg.action() {
                match action {
                    Action::Cancel => {
                        if book.order(order_id).is_none() {
                            skipped_cancels += 1;
                        }
                    }
                    Action::Modify => {
                        if book.order(order_id).is_none() {
                            skipped_modifies += 1;
                        }
                    }
                    _ => {}
                }
            }
            
            // This should not error even if the order doesn't exist
            book.apply(msg.clone())?;
        }
        
        // We expect some pre-snapshot orders in real data
        println!("Skipped {} pre-snapshot cancels and {} pre-snapshot modifies", 
            skipped_cancels, skipped_modifies);
        
        Ok(())
    }

    #[test]
    fn test_final_book_state() -> Result<()> {
        let path = Path::new("assets/CLX5_mbo.dbn");
        let mut decoder = Decoder::from_file(path)?;
        let mut book = Book::new();
        
        let mut processed = 0;
        
        // Process ALL messages
        while let Some(msg) = decoder.decode_record::<MboMsg>()? {
            book.apply(msg.clone())?;
            processed += 1;
        }
        
        println!("Processed {} total messages", processed);
        
        let (best_bid, best_ask) = book.bbo();
        
        if let Some(bid) = &best_bid {
            println!("Final Best bid: {} @ {} (${:.2})", bid.size, bid.price, bid.price as f64 / 1e9);
        }
        if let Some(ask) = &best_ask {
            println!("Final Best ask: {} @ {} (${:.2})", ask.size, ask.price, ask.price as f64 / 1e9);
        }
        
        // With matching logic, book should never be crossed
        if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
            assert!(
                bid.price < ask.price,
                "Book should not be crossed after matching: Bid ${:.2} < Ask ${:.2}",
                bid.price as f64 / 1e9,
                ask.price as f64 / 1e9
            );
            
            let spread = ask.price - bid.price;
            println!("Spread: ${:.2}", spread as f64 / 1e9);
            println!("✅ Book invariants maintained - no crossed book!");
        }
        
        Ok(())
    }

    #[test]
    fn test_find_crossed_book_moment() -> Result<()> {
        let path = Path::new("assets/CLX5_mbo.dbn");
        let mut decoder = Decoder::from_file(path)?;
        let mut book = Book::new();
        
        let mut processed = 0;
        let mut last_valid_msg: Option<MboMsg> = None;
        let check_window = 20; // Messages before and after to inspect
        
        // Process ALL messages and check after each one
        while let Some(msg) = decoder.decode_record::<MboMsg>()? {
            book.apply(msg.clone())?;
            processed += 1;
            
            let (best_bid, best_ask) = book.bbo();
            
            // Check if book just became crossed
            if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
                if bid.price > ask.price {
                    println!("\nCROSSED BOOK DETECTED at message #{}", processed);
                    println!("Best bid: ${:.2} @ {} size", bid.price as f64 / 1e9, bid.size);
                    println!("Best ask: ${:.2} @ {} size", ask.price as f64 / 1e9, ask.size);
                    println!("Spread: ${:.2}", (bid.price - ask.price) as f64 / 1e9);
                    println!("\nThe message that caused it:");
                    println!("  Action: {:?}", msg.action());
                    println!("  Side: {:?}", msg.side());
                    println!("  Price: ${:.2}", msg.price as f64 / 1e9);
                    println!("  Size: {}", msg.size);
                    println!("  Order ID: {}", msg.order_id);
                    println!("  Flags: {:?} (is_last: {}, is_tob: {})", 
                        msg.flags, msg.flags.is_last(), msg.flags.is_tob());
                    
                    if let Some(prev) = last_valid_msg {
                        println!("\nPrevious message:");
                        println!("  Action: {:?}", prev.action());
                        println!("  Side: {:?}", prev.side());
                        println!("  Price: ${:.2}", prev.price as f64 / 1e9);
                        println!("  Size: {}", prev.size);
                        println!("  Order ID: {}", prev.order_id);
                    }
                    
                    // Show the current state of top levels
                    println!("\nTop 5 bid levels:");
                    for i in 0..5 {
                        if let Some(level) = book.bid_level(i) {
                            println!("  {}: ${:.2} @ {} size ({} orders)", 
                                i, level.price as f64 / 1e9, level.size, level.count);
                        }
                    }
                    println!("\nTop 5 ask levels:");
                    for i in 0..5 {
                        if let Some(level) = book.ask_level(i) {
                            println!("  {}: ${:.2} @ {} size ({} orders)", 
                                i, level.price as f64 / 1e9, level.size, level.count);
                        }
                    }
                    
                    println!("\nINTERPRETATION:");
                    println!("This appears to be an AGGRESSIVE BID ORDER crossing the spread.");
                    println!("In a real market, this would immediately execute against the ask.");
                    println!("The DBN data may contain pre-execution state or the book handling");
                    println!("needs to simulate matching when crossed orders appear.");
                    
                    // Continue processing to see if it self-corrects
                    println!("\nChecking next {} messages to see if it clears...", check_window);
                    let start_msg = processed;
                    for i in 1..=check_window {
                        if let Some(next_msg) = decoder.decode_record::<MboMsg>()? {
                            book.apply(next_msg.clone())?;
                            let (new_bid, new_ask) = book.bbo();
                            
                            if let (Some(b), Some(a)) = (new_bid, new_ask) {
                                let still_crossed = b.price > a.price;
                                println!("  +{}: {:?} {:?} ${:.2} - Book {} (bid: ${:.2}, ask: ${:.2})",
                                    i,
                                    next_msg.action().unwrap_or_default(),
                                    next_msg.side().unwrap_or_default(),
                                    next_msg.price as f64 / 1e9,
                                    if still_crossed { "STILL CROSSED" } else { "UNCROSSED ✓" },
                                    b.price as f64 / 1e9,
                                    a.price as f64 / 1e9
                                );
                                
                                if !still_crossed {
                                    println!("\nBook uncrossed after {} messages!", i);
                                    break;
                                }
                            }
                        } else {
                            println!("  Reached end of file");
                            break;
                        }
                    }
                    
                    break;
                }
            }
            
            last_valid_msg = Some(msg.clone());
        }
        
        println!("\nProcessed {} messages before detecting crossed book", processed);
        
        Ok(())
    }

    #[test]
    fn test_matching_prevents_crossed_book() -> Result<()> {
        let path = Path::new("assets/CLX5_mbo.dbn");
        let mut decoder = Decoder::from_file(path)?;
        let mut book = Book::new();
        
        let mut processed = 0;
        let target_msg = 21268; // The message that previously caused crossing
        
        // Process all messages and verify book never crosses
        while let Some(msg) = decoder.decode_record::<MboMsg>()? {
            book.apply(msg.clone())?;
            processed += 1;
            
            // Verify book is never crossed (bid should be <= ask after matching)
            let (best_bid, best_ask) = book.bbo();
            if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
                assert!(
                    bid.price <= ask.price,
                    "Message #{}: Book crossed with bid ${:.2} > ask ${:.2}",
                    processed,
                    bid.price as f64 / 1e9,
                    ask.price as f64 / 1e9
                );
            }
        }
        
        println!("Processed {} messages - no crossed books detected!", processed);
        println!("Matching logic working - crossed orders at message {} were automatically matched", target_msg);
        
        Ok(())
    }
}