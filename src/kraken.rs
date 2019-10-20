use futures_async_stream::async_stream;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;
use crate::trade::*;

pub struct Kraken {
    pub base_url: String,
}

impl Kraken {
    pub async fn pub_api(
        &self,
        endpoint: String,
        query: HashMap<String, String>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/{}", self.base_url, endpoint);

        Ok(reqwest::Client::new()
            .get(&url)
            .query(&query)
            .send()
            .await?
            .json()
            .await?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KrakenTrade(String, String, f64, String, String, String);

impl From<KrakenTrade> for Trade {
    fn from(trade: KrakenTrade) -> Self {
        Trade {
            symbol: TradeSymbol {
                base_currency: "ETH".to_string(),
                quote_currency: "EUR".to_string(),
            },
            price: trade.0.parse::<f64>().unwrap(),
            volume: trade.1.parse::<f64>().unwrap(),
            timestamp: (trade.2 * 10_000.0) as u64,
        }
    }
}

impl TradeAPI for Kraken {
    // #[async_try_stream(boxed, ok = Trade, error = Box<dyn std::error::Error + Send + Sync>)]
    #[async_stream(boxed, item = Trade)]
    async fn history(&self, symbol: TradeSymbol) {
        let mut query = HashMap::new();
        query.insert("pair".to_string(), "ETHEUR".to_string());
        let mut res: Value = self.pub_api("Trades".to_string(), query).await.expect("Couldn't fetch history");
        let trades: Vec<KrakenTrade> = serde_json::from_value(res["result"]["XETHZEUR"].take()).unwrap();
        
        for trade in trades {
            yield Trade::from(trade);
        }
    }
}
