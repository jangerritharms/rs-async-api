use crate::api::*;
use crate::trade::*;
use futures::{stream, Stream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Kraken<Client: HTTPRequest> {
    pub client: Client,
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
    fn from(res: KrakenResponse<HistoryResponse>) -> Self {
        let mut result = res.result.unwrap();
        let trades: Vec<KrakenTrade> = result.trades.remove("XETHZEUR").unwrap();
        PaginationHelper {
            trades: trades.into_iter().map(|trade| Trade::from(trade)).collect(),
            continuation: result.last.parse::<u64>().unwrap(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KrakenResponse<T> {
    error: Vec<String>,
    result: Option<T>,
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

#[derive(Debug,PartialEq,Deserialize)]
pub struct Asset {
    altname: String,
    aclass: String,
    decimals: u8,
    display_decimals: u8,
}

type AssetResponse = HashMap<String,Asset>;

impl<Client: HTTPRequest> Kraken<Client> {
    pub async fn assets(
        &self,
    ) -> std::result::Result<KrakenResponse<AssetResponse>, Box<dyn std::error::Error>> {
        let req: HashMap<String, String> = HashMap::new();
        self.client
            .req(String::from("Assets"), req)
            .await
            .map_err(|e| e.into())
            .map(|res| serde_json::from_str(&res).unwrap())
    }

    pub async fn history(
        &self,
        _symbol: &TradeSymbol,
        since: u64,
    ) -> std::result::Result<KrakenResponse<HistoryResponse>, Box<dyn std::error::Error>> {
        self.client
            .req(
                String::from("Trades"),
                HistoryRequest {
                    pair: "ETHEUR".to_string(),
                    since,
                },
            )
            .await
            .map_err(|e| e.into())
            .map(|res| serde_json::from_str(&res).unwrap())
    }

    pub fn history_since_until_now(
        &self,
        _symbol: TradeSymbol,
        since: u64,
    ) -> impl Stream<Item = Trade> + '_ {
        let init = PaginationHelper {
            trades: Vec::new(),
            continuation: since,
        };
        stream::unfold(init, move |mut state: PaginationHelper| {
            async move {
                if state.trades.len() == 0 {
                    let symbol = TradeSymbol {
                        base_currency: "ETH".to_string(),
                        quote_currency: "EUR".to_string(),
                    };
                    state = PaginationHelper::from(
                        self.history(&symbol, state.continuation).await.unwrap(),
                    );
                    if state.trades.len() == 0 {
                        return None;
                    }
                }
                let yielded = match state.trades.pop() {
                    Some(trade) => trade,
                    None => return None,
                };
                Some((yielded, state))
            }
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
        assert_eq!(history.result.unwrap().trades.len(), 1);
    }

    use async_trait::async_trait;
    use std::error::Error;
    use tokio::runtime::current_thread::Runtime;

    struct FakeClient {
        response: String,
    }

    #[async_trait]
    impl HTTPRequest for FakeClient {
        async fn req<Req>(
            &self,
            _endpoint: String,
            _query: Req,
        ) -> std::result::Result<String, Box<dyn Error>>
        where
            Req: Serialize + Sized + std::marker::Sync + std::marker::Send,
        {
            Ok(self.response.clone())
        }
    }

    #[test]
    fn test_fn_history() {
        let trade_json = r#"
        {        
            "error": [],
            "result": {
                "XETHZEUR": [
                    [
                        "138.65000",
                        "1.55284051",
                        1575127767.9793,
                        "b",
                        "l",
                        ""
                    ],
                    [
                        "138.66000",
                        "10.00000000",
                        1575127767.9842,
                        "b",
                        "l",
                        ""
                    ]
                ],
                "last": "1575145023655038533"
            }
        }
        "#;

        let client = FakeClient {
            response: trade_json.to_string(),
        };
        let k = Kraken { client };

        let sym = TradeSymbol {
            base_currency: "ETH".to_string(),
            quote_currency: "EUR".to_string(),
        };
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let res = k.history(&sym, 1575127767000000000).await.unwrap();
            assert_eq!(res.result.unwrap().trades["XETHZEUR"].len(), 2)
        });
    }

    #[test]
    fn test_fn_assets() {
        let assets_json = r#"
        {
            "error": [],
            "result": {
                "ADA": {
                    "aclass": "currency",
                    "altname": "ADA",
                    "decimals": 8,
                    "display_decimals": 6
                },
                "ATOM": {
                    "aclass": "currency",
                    "altname": "ATOM",
                    "decimals": 8,
                    "display_decimals": 6
                }
            }
        }
        "#;

        let client = FakeClient {
            response: assets_json.to_string(),
        };
        let k = Kraken { client };

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let res = k.assets().await.unwrap();
            let mut expected: HashMap<String,Asset> = HashMap::new();
            expected.insert("ADA".to_string(), Asset {
                aclass: "currency".to_string(),
                altname: "ADA".to_string(),
                decimals: 8,
                display_decimals: 6
            });
            expected.insert("ATOM".to_string(), Asset {
                aclass: "currency".to_string(),
                altname: "ATOM".to_string(),
                decimals: 8,
                display_decimals: 6
            });
            assert_eq!(res.result.unwrap(), expected)
        });
    }

    #[test]
    fn test_fn_error() {
        let trade_json = r#"
        {
            "error": [
                "EQuery:Unknown asset pair"
            ]
        }
        "#;

        let client = FakeClient {
            response: trade_json.to_string(),
        };
        let k = Kraken { client };

        let sym = TradeSymbol {
            base_currency: "ETH".to_string(),
            quote_currency: "EUR".to_string(),
        };
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let res = k.history(&sym, 1575127767000000000).await.unwrap();
            assert_eq!(res.error[0], String::from("EQuery:Unknown asset pair"))
        });
    }
}
