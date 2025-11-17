use std::path::Path;

use dbn::{
    decode::{DecodeRecord, dbn::Decoder},
    record::MboMsg
};
use anyhow::{Result, Context, ensure};

#[tokio::main]
async fn main() -> Result<()> {
    // First, check that the file exists
    let input_file_path = Path::new("assets/CLX5_mbo.dbn");
    ensure!(input_file_path.exists(), "Input file does not exist at path: `{:?}`", input_file_path);

    let mut dbn_stream = Decoder::from_file(input_file_path)
        .context("...while trying to open decoder on file")?;
    
    while let Some(mbo_msg) = dbn_stream.decode_record::<MboMsg>().context("...while trying to decode record")? {
        println!("{mbo_msg:?}");
    }

    Ok(())
}