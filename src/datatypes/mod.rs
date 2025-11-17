pub mod book;
pub mod market;
pub mod price_level;

use std::collections::VecDeque;
use databento::dbn::MboMsg;

pub type Level = VecDeque<MboMsg>;