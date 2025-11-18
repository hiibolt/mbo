use std::collections::HashMap;
use super::{price_level::PriceLevel, book::Book};
use databento::{
    dbn::{
        MboMsg, Publisher, Record,
        decode::{DecodeRecord, dbn::Decoder, DbnMetadata},
        SymbolIndex
    }
};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use tracing::info;
use std::path::Path;
use crate::{datatypes::book::BookEffect, storage::Storage};

#[derive(Debug, Clone, Serialize)]
pub struct MarketEffect {
    pub publisher_created: Option<Publisher>,
    pub book_effect: Result<Option<BookEffect>, String>
}
impl Default for MarketEffect {
    fn default() -> Self {
        Self {
            publisher_created: None,
            book_effect: Ok(None)
        }
    }
}
impl MarketEffect {
    pub fn from_book_effect(effect: Result<Option<BookEffect>, String>) -> Self {
        Self {
            publisher_created: None,
            book_effect: effect
        }
    }

    pub fn add_publisher_created(&mut self, publisher: Publisher) {
        self.publisher_created = Some(publisher);
    }
}

/// Load market state from DBN file and return both the market and all MBO messages
/// 
/// Optionally persists messages to storage if provided.
pub fn load_market_snapshots(
    path: &Path,
    storage: Option<&Storage>,
) -> Result<Vec<MarketSnapshot>> {
    // First, check that the file exists - `Decoder::from_file`
    //  already does, but the error message isn't helpful at all
    ensure!(path.exists(), "Input file does not exist at path: `{:?}`", path);

    let mut dbn_decoder = Decoder::from_file(path)
        .context("...while trying to open decoder on file")?;
    let mut mbo_messages = Vec::new();
    let symbol_map = dbn_decoder.metadata().symbol_map()?;

    info!("File loaded, beginning to process MBO messages...");
    
    // Batch size for database inserts
    const BATCH_SIZE: usize = 1000;
    let mut batch = Vec::with_capacity(BATCH_SIZE);
    
    let mut snapshots = Vec::new();
    let mut market = Market::new();
    while let Some(mbo_msg) = dbn_decoder.decode_record::<MboMsg>().context("...while trying to decode record")? {
        // Store the message for TCP streaming
        mbo_messages.push(mbo_msg.clone());
        
        // Add to batch for persistence
        if let Some(storage) = storage {
            batch.push(mbo_msg.clone());
            
            // Persist batch when it reaches batch size
            if batch.len() >= BATCH_SIZE {
                storage.insert_mbo_batch(&batch)
                    .context("...while persisting MBO message batch")?;
                batch.clear();
            }
        }
        let market_effect = market.apply(mbo_msg.clone())
            .context("...while trying to apply MBO message to market")?;

        // Capture market snapshot after applying the MBO message
        snapshots.push(MarketSnapshot {
            market: market.clone(),
            market_effect: market_effect,
            applied_mbo_msg: mbo_msg.clone(),
        });

        // If it's the last update in an event, print the state of the aggregated book
        if mbo_msg.flags.is_last() {
            let symbol = symbol_map.get_for_rec(mbo_msg)
                .context("...while trying to get symbol for MBO message")?;
            let (best_bid, best_offer) = market.aggregated_bbo(mbo_msg.hd.instrument_id);
            
            let ts_recv = mbo_msg.ts_recv().context("...while trying to get ts_recv")?;
            info!(
                symbol = %symbol,
                timestamp = %ts_recv,
                best_bid = ?best_bid,
                best_offer = ?best_offer,
                "Aggregated BBO update"
            );
        }
    }
    
    // Persist any remaining messages in the batch
    if let Some(storage) = storage {
        if !batch.is_empty() {
            storage.insert_mbo_batch(&batch)
                .context("...while persisting final MBO message batch")?;
        }
        
        let total_count = storage.count_messages()
            .context("...while counting persisted messages")?;
        info!("Persisted {} total messages to database", total_count);
    }
    
    info!("Finished processing DBN file. Loaded {} MBO messages.", mbo_messages.len());

    Ok(snapshots)
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct MarketSnapshot {
    market: Market,
    market_effect: MarketEffect,
    applied_mbo_msg: MboMsg
}
#[derive(Debug, Default, Clone, Serialize)]
pub struct Market {
    books: HashMap<u32, Vec<(Publisher, Book)>>,
}
impl Market {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn books_by_pub(&self, instrument_id: u32) -> Option<&[(Publisher, Book)]> {
        self.books
            .get(&instrument_id)
            .map(|pub_books| pub_books.as_slice())
    }

    #[tracing::instrument(skip(self))]
    pub fn aggregated_bbo(&self, instrument_id: u32) -> (Option<PriceLevel>, Option<PriceLevel>) {
        let mut agg_bid = None;
        let mut agg_ask = None;
        let Some(books_by_pub) = self.books_by_pub(instrument_id) else {
            return (None, None);
        };
        for (_, book) in books_by_pub.iter() {
            let (bid, ask) = book.bbo();
            if let Some(bid) = bid {
                match &mut agg_bid {
                    None => agg_bid = Some(bid),
                    Some(ab) if bid.price > ab.price => agg_bid = Some(bid),
                    Some(ab) if bid.price == ab.price => {
                        ab.size += bid.size;
                        ab.count += bid.count;
                    }
                    Some(_) => {}
                }
            }
            if let Some(ask) = ask {
                match &mut agg_ask {
                    None => agg_ask = Some(ask),
                    Some(aa) if ask.price < aa.price => agg_ask = Some(ask),
                    Some(aa) if ask.price == aa.price => {
                        aa.size += ask.size;
                        aa.count += ask.count;
                    }
                    Some(_) => {}
                }
            }
        }
        (agg_bid, agg_ask)
    }

    #[tracing::instrument(skip(self), fields(instrument_id = mbo.hd.instrument_id, order_id = mbo.order_id))]
    pub fn apply(&mut self, mbo: MboMsg) -> Result<MarketEffect> {
        let publisher = mbo.publisher()
            .context("MBO message has no valid publisher")?;
        let books = self.books.entry(mbo.hd.instrument_id).or_default();
        let mut created_publisher = None;
        let book = if let Some((_, book)) = books
            .iter_mut()
            .find(|(book_pub, _)| *book_pub == publisher)
        {
            book
        } else {
            books.push((publisher.clone(), Book::default()));
            created_publisher = Some(publisher.clone());
            &mut books
                .last_mut()
                .context("Books vector is unexpectedly empty after push")?
                .1
        };

        let book_effect = book.apply(mbo.clone())
            .context("...while applying MBO message to book")?;
        let mut market_effect = MarketEffect::from_book_effect(book_effect);
        if let Some(pub_created) = created_publisher {
            market_effect.add_publisher_created(pub_created);
        }

        Ok(market_effect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_market_with_real_data() -> Result<()> {
        let path = Path::new("assets/CLX5_mbo.dbn");
        
        // Load market from the real DBN file (without storage to keep test simple)
        let market_snapshots = load_market_snapshots(path, None)?;
        
        println!("Loaded {} market snapshots from DBN file.", 
            market_snapshots.len(), 
        );
        
        Ok(())
    }
}