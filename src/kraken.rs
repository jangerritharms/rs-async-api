use crate::api::*;
use crate::error::Error;
use crate::trade::*;
use futures::{stream, Stream};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Kraken<Client: HTTPRequest> {
    pub client: Client,
}

#[derive(Serialize, PartialEq, Deserialize, Debug)]
pub struct KrakenTrade(String, String, f64, String, String, String);

impl From<KrakenTrade> for Trade {
    fn from(trade: KrakenTrade) -> Self {
        Trade {
            pair: "ETHEUR".to_string(),
            price: trade.0.parse::<f64>().unwrap(),
            volume: trade.1.parse::<f64>().unwrap(),
            timestamp: (trade.2 * 10_000.0) as u64,
        }
    }
}

#[derive(Debug)]
struct PaginationHelper {
    pair: String,
    trades: Vec<Trade>,
    continuation: u64,
}

impl From<HistoryResponse> for PaginationHelper {
    fn from(mut res: HistoryResponse) -> Self {
        let mut pair: String = "".to_string();
        for key in res.trades.keys() {
            if key != "last" {
                pair = key.to_string();
                break;
            }
        }
        let trades: Vec<KrakenTrade> = res.trades.remove(&pair).unwrap();
        PaginationHelper {
            pair,
            trades: trades.into_iter().map(|trade| Trade::from(trade)).collect(),
            continuation: res.last.parse::<u64>().unwrap(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KrakenResponse<T> {
    error: Vec<String>,
    result: Option<T>,
}

#[derive(Debug, PartialEq, Deserialize)]
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

#[derive(Debug, PartialEq, Deserialize)]
pub struct Asset {
    altname: String,
    aclass: String,
    decimals: u8,
    display_decimals: u8,
}

type AssetResponse = HashMap<String, Asset>;

impl<Client: HTTPRequest> Kraken<Client> {
    pub async fn req<Req, Res>(&self, endpoint: String, query: Req) -> Result<Res, Error>
    where
        Req: Serialize + Sized + std::marker::Sync + std::marker::Send,
        Res: DeserializeOwned,
    {
        self.client
            .req(endpoint, query)
            .await
            .and_then(|res| serde_json::from_str::<KrakenResponse<Res>>(&res).map_err(|e| e.into()))
            .and_then(|res| match res.result {
                Some(result) => Ok(result),
                None => Err(Error::KrakenError(res.error)),
            })
    }

    pub async fn assets(&self) -> Result<AssetResponse, Error> {
        let req: HashMap<String, String> = HashMap::new();
        self.req(String::from("Assets"), req).await
    }

    pub async fn history(
        &self,
        pair: String,
        since: u64,
    ) -> std::result::Result<HistoryResponse, Error> {
        self.req(String::from("Trades"), HistoryRequest { pair, since })
            .await
    }

    pub fn history_since_until_now(
        &self,
        pair: String,
        since: u64,
    ) -> impl Stream<Item = Trade> + '_ {
        let init = PaginationHelper {
            pair,
            trades: Vec::new(),
            continuation: since,
        };
        stream::unfold(init, move |mut state: PaginationHelper| {
            async move {
                if state.trades.len() == 0 {
                    state = PaginationHelper::from(
                        self.history(state.pair, state.continuation).await.unwrap(),
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
        ) -> std::result::Result<String, Error>
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

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let res = k.history("ETHEUR".to_string(), 1575127767000000000).await.unwrap();
            assert_eq!(res.trades["XETHZEUR"].len(), 2)
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
            let res = k.assets().await;
            let mut expected: HashMap<String, Asset> = HashMap::new();
            expected.insert(
                "ADA".to_string(),
                Asset {
                    aclass: "currency".to_string(),
                    altname: "ADA".to_string(),
                    decimals: 8,
                    display_decimals: 6,
                },
            );
            expected.insert(
                "ATOM".to_string(),
                Asset {
                    aclass: "currency".to_string(),
                    altname: "ATOM".to_string(),
                    decimals: 8,
                    display_decimals: 6,
                },
            );
            assert_eq!(res, Ok(expected))
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

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            let res = k.history("ETHEUR".to_string(), 1575127767000000000).await;
            assert_eq!(
                res,
                Err(Error::KrakenError(vec!(String::from(
                    "EQuery:Unknown asset pair"
                ))))
            )
        });
    }
}
