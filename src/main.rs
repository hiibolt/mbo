use std::path::Path;

use dbn::{
    decode::{DecodeRecord, dbn::Decoder},
    record::MboMsg
};
use anyhow::{Result, Context, ensure};
use tracing::info;

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

    // First, check that the file exists
    let input_file_path = Path::new("assets/CLX5_mbo.dbn");
    ensure!(input_file_path.exists(), "Input file does not exist at path: `{:?}`", input_file_path);

    let mut dbn_stream = Decoder::from_file(input_file_path)
        .context("...while trying to open decoder on file")?;
    
    while let Some(mbo_msg) = dbn_stream.decode_record::<MboMsg>().context("...while trying to decode record")? {
        println!("{mbo_msg:?}");
    }

    info!("Finished processing DBN file.");

    Ok(())
}