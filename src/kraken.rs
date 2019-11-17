use futures::{stream, Stream};
use futures::stream::{StreamExt};
use reqwest::Result;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use serde_json::Value;
use crate::trade::*;

pub struct Kraken {
    pub base_url: String,
}

impl Kraken {
    pub async fn pub_api<Req, Res>(
        &self,
        endpoint: String,
        query: Req,
    ) -> Result<Res>
        where 
        Req: Serialize + Sized,
        Res: DeserializeOwned {
        let url = format!("{}/{}", self.base_url, endpoint);

        Ok(reqwest::Client::new()
            .get(&url)
            .query(&query)
            .send()
            .await?
            .json::<Res>()
            .await?
        )
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
struct PaginationHelper {
    trades: Vec<Trade>,
    continuation: u64,
}

impl From<KrakenResponse<HistoryResponse>> for PaginationHelper {
    fn from(mut res: KrakenResponse<HistoryResponse>) -> Self {
        let trades: Vec<KrakenTrade> = res.result.trades.remove("XETHZEUR").unwrap();
        PaginationHelper {
            trades: trades.into_iter().map(|trade| Trade::from(trade)).collect(),
            continuation: res.result.last.parse::<u64>().unwrap(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KrakenResponse<T> {
    error: Vec<String>,
    result: T,
}

#[derive(Debug, Deserialize)]
pub struct HistoryResponse {
    #[serde(flatten)]
    trades: HashMap<String, Vec<KrakenTrade>>,
    last: String,
}

type Symbol = String;

#[derive(Serialize)]
pub struct HistoryRequest {
    pair: Symbol,
    since: u64,
}


impl Kraken {
    pub async fn history(&self, symbol: &TradeSymbol, since: u64) -> Result<KrakenResponse<HistoryResponse>> {
        self.pub_api(String::from("Trades"), HistoryRequest {
            pair: "ETHEUR".to_string(),
            since
        }).await
    }

    pub fn history_since_until_now(&self, symbol: TradeSymbol, since: u64) -> impl Stream<Item = Trade> + '_ {
        let init = PaginationHelper {
            trades: Vec::new(),
            continuation: since,
        };
        stream::unfold(init, move |mut state: PaginationHelper| async move {
            if state.trades.len() == 0 {
                let symbol = TradeSymbol {
                    base_currency: "ETH".to_string(),
                    quote_currency: "EUR".to_string(),
                };
                state = PaginationHelper::from(self.history(&symbol, state.continuation).await.unwrap());
                if (state.trades.len() == 0) {
                    return None
                } 
            }
            let yielded = match state.trades.pop() { Some(trade) => trade,
                None => return None
            };
            Some((yielded, state))
        })
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_kraken_trade_json() {
        let trade_json = r#"
        [
            "164.70000",
            "2.00000000",
            1573915002.1839,
            "s",
            "m",
            ""
        ]
        "#;

        let trade: KrakenTrade = serde_json::from_str(trade_json).unwrap();
        assert_eq!(trade.1, "2.00000000");
    }

    #[test]
    fn test_history_json() {
        let trade_json = r#"
        {
            "XETHZEUR": [
                [
                    "165.39000",
                    "0.02022600",
                    1573919178.4009,
                    "b",
                    "l",
                    ""
                ]
            ],
            "last": "1573919178400857031"
        }
        "#;

        let history: HistoryResponse = serde_json::from_str(trade_json).unwrap();
        assert_eq!(history.trades.len(), 1);
    }

    #[test]
    fn test_history_response_json() {
        let trade_json = r#"
        {
            "error": [],
            "result": {
                "XETHZEUR": [
                    [
                        "165.39000",
                        "0.02022600",
                        1573919178.4009,
                        "b",
                        "l",
                        ""
                    ]
                ],
                "last": "1573919178400857031"
            }
        }
        "#;

        let history: KrakenResponse<HistoryResponse> = serde_json::from_str(trade_json).unwrap();
        assert_eq!(history.result.trades.len(), 1);
    }
}
