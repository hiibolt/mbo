use std::collections::{BTreeMap, HashMap, VecDeque};

use super::{price_level::PriceLevel, Level};
use databento::{
    dbn::{
        Action, BidAskPair, MboMsg, Side,
        UNDEF_PRICE,
    },
};
use anyhow::{Result, Context, bail};

#[derive(Debug, Default)]
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

    pub fn apply(&mut self, mbo: MboMsg) -> Result<()> {
        let action = mbo.action()
            .context("Failed to get action from MBO message")?;
        match action {
            Action::Modify => self.modify(mbo)?,
            Action::Trade | Action::Fill | Action::None => {}
            Action::Cancel => self.cancel(mbo)?,
            Action::Add => self.add(mbo)?,
            Action::Clear => self.clear(),
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.orders_by_id.clear();
        self.offers.clear();
        self.bids.clear();
    }

    fn add(&mut self, mbo: MboMsg) -> Result<()> {
        let price = mbo.price;
        let side = mbo.side()
            .context("Failed to get side from MBO message")?;
        if mbo.flags.is_tob() {
            let levels: &mut BTreeMap<i64, Level> = self.side_levels_mut(side)?;
            levels.clear();
            // UNDEF_PRICE indicates the side's book should be cleared
            // and doesn't represent an order that should be added
            if mbo.price != UNDEF_PRICE {
                levels.insert(price, VecDeque::from([mbo]));
            }
        } else {
            assert_ne!(price, UNDEF_PRICE);
            assert!(self
                .orders_by_id
                .insert(mbo.order_id, (side, price))
                .is_none());
            let level: &mut Level = self.get_or_insert_level(side, price)?;
            level.push_back(mbo);
        }
        Ok(())
    }

    fn cancel(&mut self, mbo: MboMsg) -> Result<()> {
        let side = mbo.side()
            .context("Failed to get side from MBO message")?;
        let level = self.level_mut(side, mbo.price)?;
        let order_idx = Self::find_order(level, mbo.order_id)
            .context("Failed to find order in level")?;
        let existing_order = level.get_mut(order_idx)
            .context("Failed to get order at index")?;
        assert!(existing_order.size >= mbo.size);
        existing_order.size -= mbo.size;
        if existing_order.size == 0 {
            level.remove(order_idx)
                .context("Failed to remove order from level")?;
            if level.is_empty() {
                self.remove_level(side, mbo.price)?;
            }
            self.orders_by_id.remove(&mbo.order_id)
                .context("Failed to remove order from orders_by_id map")?;
        }
        Ok(())
    }

    fn modify(&mut self, mbo: MboMsg) -> Result<()> {
        let order_id = mbo.order_id;
        let side = mbo.side()
            .context("Failed to get side from MBO message")?;
        let Some((id_side, id_price)) = self.orders_by_id.get_mut(&order_id) else {
            // If order not found, treat it as an add
            return self.add(mbo);
        };
        let prev_side = *id_side;
        let prev_price = *id_price;
        // Update orders by ID
        *id_side = side;
        *id_price = mbo.price;
        // Update level order
        let level = self.level_mut(prev_side, prev_price)?;
        let order_idx = Self::find_order(level, mbo.order_id)
            .context("Failed to find order in level")?;
        let existing_order = level.get_mut(order_idx)
            .context("Failed to get order at index")?;
        existing_order.size = mbo.size;
        let should_keep_priority = prev_price == mbo.price && existing_order.size >= mbo.size;
        if should_keep_priority {
            return Ok(());
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
        Ok(())
    }

    fn get_or_insert_level(&mut self, side: Side, price: i64) -> Result<&mut Level> {
        let levels = self.side_levels_mut(side)?;
        Ok(levels.entry(price).or_default())
    }

    fn level_mut(&mut self, side: Side, price: i64) -> Result<&mut Level> {
        let levels = self.side_levels_mut(side)?;
        levels.get_mut(&price)
            .context(format!("Failed to get level at price {}", price))
    }

    fn remove_level(&mut self, side: Side, price: i64) -> Result<()> {
        self.side_levels_mut(side)?
            .remove(&price)
            .context(format!("Failed to remove level at price {}", price))?;
        Ok(())
    }

    fn find_order(level: &VecDeque<MboMsg>, order_id: u64) -> Result<usize> {
        level.iter()
            .position(|o| o.order_id == order_id)
            .context(format!("Failed to find order with ID {}", order_id))
    }

    fn remove_order(level: &mut VecDeque<MboMsg>, order_id: u64) -> Result<()> {
        let index = Self::find_order(level, order_id)?;
        level.remove(index)
            .context(format!("Failed to remove order at index {}", index))?;
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