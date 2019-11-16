use futures::{stream, Stream};
use futures::stream::{StreamExt};
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

#[derive(Debug)]
struct ResponseStruct {
    trades: Vec<KrakenTrade>,
    contTimestamp: String,
}

impl Kraken {
    pub fn history_since_until_now(&self, symbol: TradeSymbol, since: u64) -> impl Stream<Item = KrakenTrade> + '_ {
        let init = ResponseStruct {
            trades: Vec::new(),
            contTimestamp: since.to_string(),
        };
        stream::unfold(init, move |mut state: ResponseStruct| async move {
            if state.trades.len() == 0 {
                let mut query = HashMap::new();
                query.insert(String::from("pair"), String::from("ETHEUR"));
                query.insert(String::from("since"), state.contTimestamp.clone());
                let mut res: Value = self.pub_api(String::from("Trades"), query).await.expect("Couldn't fetch history");
                state = ResponseStruct {
                    trades: serde_json::from_value(res["result"]["XETHZEUR"].take()).unwrap(),
                    contTimestamp: serde_json::from_value(res["result"]["last"].take()).unwrap(),
                };
                if (state.trades.len() == 0) {
                    return None
                } 
            }
            let yielded = match state.trades.pop() {
                Some(trade) => trade,
                None => return None
            };
            Some((yielded, state))
        })
    }
}
