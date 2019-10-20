#![feature(stmt_expr_attributes, proc_macro_hygiene)]
use crytrade::kraken;
use crytrade::trade;
use crytrade::trade::{TradeAPI};
use futures_async_stream::for_await;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let kraken = kraken::Kraken {
        base_url: "https://api.kraken.com/0/public".to_string(),
    };
    let sym = trade::TradeSymbol {
        base_currency: "ETH".to_string(),
        quote_currency: "EUR".to_string(),
    };
    #[for_await]
    for value in kraken.history(sym) {
        println!("{:#?}", value);
    }

    Ok(())
}
