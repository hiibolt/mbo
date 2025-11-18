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
use crate::storage::Storage;

#[derive(Debug, Default, Serialize)]
pub struct Market {
    books: HashMap<u32, Vec<(Publisher, Book)>>,
}
impl Market {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the count of instruments in the market
    pub fn instrument_count(&self) -> usize {
        self.books.len()
    }

    /// Load market state from DBN file and return both the market and all MBO messages
    /// 
    /// Optionally persists messages to storage if provided.
    pub fn load_from_path_with_messages(
        path: &Path,
        storage: Option<&Storage>,
    ) -> Result<(Self, Vec<MboMsg>)> {
        // First, check that the file exists - `Decoder::from_file`
        //  already does, but the error message isn't helpful at all
        ensure!(path.exists(), "Input file does not exist at path: `{:?}`", path);

        let mut dbn_decoder = Decoder::from_file(path)
            .context("...while trying to open decoder on file")?;
        let mut market = Market::new();
        let mut mbo_messages = Vec::new();
        let symbol_map = dbn_decoder.metadata().symbol_map()?;

        info!("File loaded, beginning to process MBO messages...");
        
        // Batch size for database inserts
        const BATCH_SIZE: usize = 1000;
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        
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
            
            market.apply(mbo_msg.clone())
                .context("...while trying to apply MBO message to market")?;

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
    
        Ok((market, mbo_messages))
    }

    pub fn books_by_pub(&self, instrument_id: u32) -> Option<&[(Publisher, Book)]> {
        self.books
            .get(&instrument_id)
            .map(|pub_books| pub_books.as_slice())
    }

    pub fn book(&self, instrument_id: u32, publisher: Publisher) -> Option<&Book> {
        let books = self.books.get(&instrument_id)?;
        books.iter().find_map(|(book_pub, book)| {
            if *book_pub == publisher {
                Some(book)
            } else {
                None
            }
        })
    }

    pub fn bbo(
        &self,
        instrument_id: u32,
        publisher: Publisher,
    ) -> (Option<PriceLevel>, Option<PriceLevel>) {
        self.book(instrument_id, publisher)
            .map(|book| book.bbo())
            .unwrap_or_default()
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
    pub fn apply(&mut self, mbo: MboMsg) -> Result<()> {
        let publisher = mbo.publisher()
            .context("MBO message has no valid publisher")?;
        let books = self.books.entry(mbo.hd.instrument_id).or_default();
        let book = if let Some((_, book)) = books
            .iter_mut()
            .find(|(book_pub, _)| *book_pub == publisher)
        {
            book
        } else {
            books.push((publisher, Book::default()));
            &mut books
                .last_mut()
                .context("Books vector is unexpectedly empty after push")?
                .1
        };
        book.apply(mbo)
            .context("...while applying MBO message to book")?;
        Ok(())
    }
}