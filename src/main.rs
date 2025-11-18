mod datatypes;

use std::path::Path;

use databento::{
    dbn::{
        decode::{DecodeRecord, dbn::Decoder, DbnMetadata},
        MboMsg, SymbolIndex
    },
    HistoricalClient,
};
use anyhow::{Result, Context, ensure};
use tracing::info;

use self::datatypes::market::Market;


struct Config {
    dbn_client: HistoricalClient,
}
impl Config {
    #[tracing::instrument]
    fn from_env() -> Result<Self> {
        let dbn_api_key = std::env::var("DBN_KEY")
            .context("DBN_KEY environment variable not set")?;

        let dbn_client = HistoricalClient::builder()
            .key(dbn_api_key)
            .context("...while building DBN client")?
            .build()
            .context("...while building DBN client")?;

        Ok(Self {
            dbn_client
        })
    }
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    // Initialize the logger - to view all logs, set the 
    //  RUST_LOG environment variable to "debug" or "trace"
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize configuration from environment variables
    let _config = Config::from_env()
        .context("...while loading configuration from environment")?;

    // First, check that the file exists
    let input_file_path = {
        let path = Path::new("assets/CLX5_mbo.dbn"); // Linux format is ok, `Path` handles it
        ensure!(path.exists(), "Input file does not exist at path: `{:?}`", path);
        path
    };

    let mut dbn_decoder = Decoder::from_file(input_file_path)
        .context("...while trying to open decoder on file")?;
    let mut market = Market::new();
    let symbol_map = dbn_decoder.metadata().symbol_map()?;
    while let Some(mbo_msg) = dbn_decoder.decode_record::<MboMsg>().context("...while trying to decode record")? {
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

    info!("Finished processing DBN file.");

    Ok(())
}