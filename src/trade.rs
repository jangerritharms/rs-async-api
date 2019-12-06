pub trait TradeAPI {
    fn history(&self, pair: String, since: u64);
}


#[derive(Debug)]
pub struct Trade {
    pub pair: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: u64,
}

