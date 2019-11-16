pub trait TradeAPI {
    fn history(&self, symbol: TradeSymbol);
}

#[derive(Debug)]
pub struct TradeSymbol {
    pub base_currency: String,
    pub quote_currency: String,
}

#[derive(Debug)]
pub struct Trade {
    pub symbol: TradeSymbol,
    pub price: f64,
    pub volume: f64,
    pub timestamp: u64,
}

