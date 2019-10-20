use futures_async_stream::async_stream;

pub trait TradeAPI {
    #[async_stream(boxed, item = Trade)]
    // #[async_try_stream(boxed, ok = Trade, error = Box<dyn std::error::Error + Send + Sync>)]
    async fn history(&self, symbol: TradeSymbol);
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

