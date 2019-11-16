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
    pub async fn pub_api<T>(
        &self,
        endpoint: String,
        query: HashMap<String, String>,
    ) -> Result<T>
        where T: DeserializeOwned {
        let url = format!("{}/{}", self.base_url, endpoint);

        Ok(reqwest::Client::new()
            .get(&url)
            .query(&query)
            .send()
            .await?
            .json::<T>()
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
struct ResponseStruct {
    trades: Vec<KrakenTrade>,
    contTimestamp: String,
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

impl Kraken {
    pub async fn history(&self, symbol: TradeSymbol, since: u64) -> Result<KrakenResponse<HistoryResponse>> {
        let mut query = HashMap::new();
        query.insert(String::from("pair"), String::from("ETHEUR"));
        query.insert(String::from("since"), since.to_string());
        self.pub_api(String::from("Trades"), query).await
    }

    // pub fn history_since_until_now(&self, symbol: TradeSymbol, since: u64) -> impl Stream<Item = KrakenTrade> + '_ {
    //     let init = ResponseStruct {
    //         trades: Vec::new(),
    //         contTimestamp: since.to_string(),
    //     };
    //     stream::unfold(init, move |mut state: ResponseStruct| async move {
    //         if state.trades.len() == 0 {
    //             let mut query = HashMap::new();
    //             query.insert(String::from("pair"), String::from("ETHEUR"));
    //             query.insert(String::from("since"), state.contTimestamp.clone());
    //             let mut res: HistoryResponse = self.pub_api(String::from("Trades"), query).await.expect("Couldn't fetch history");
    //             state = ResponseStruct {
    //                 trades: res,
    //                 contTimestamp: serde_json::from_value(res["result"]["last"].take()).unwrap(),
    //             };
    //             if (state.trades.len() == 0) {
    //                 return None
    //             } 
    //         }
    //         let yielded = match state.trades.pop() {
    //             Some(trade) => trade,
    //             None => return None
    //         };
    //         Some((yielded, state))
    //     })
    // }
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
