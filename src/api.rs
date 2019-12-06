use crate::error::Error;
use async_trait::async_trait;
use serde::Serialize;

pub struct KrakenClient {
    pub base_url: String,
}

#[async_trait]
pub trait HTTPRequest {
    async fn req<Req>(&self, endpoint: String, query: Req) -> Result<String, Error>
    where
        Req: Serialize + Sized + std::marker::Sync + std::marker::Send;
}

#[async_trait]
impl HTTPRequest for KrakenClient {
    async fn req<Req>(&self, endpoint: String, query: Req) -> Result<String, Error>
    where
        Req: Serialize + Sized + std::marker::Sync + std::marker::Send,
    {
        let url = format!("{}/{}", self.base_url, endpoint);

        reqwest::Client::new()
            .get(&url)
            .query(&query)
            .send()
            .await?
            .text()
            .await
            .map_err(|err| err.into())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::collections::HashMap;
    use tokio::runtime::current_thread::Runtime;
    use crate::error;

    #[test]
    fn test_kraken_client_req() {
        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            let c = KrakenClient {
                base_url: "http://httpbin.org".to_string(),
            };

            let query: HashMap<String, String> = HashMap::new();
            let res: String = c.req("get".to_string(), query).await.unwrap();
            assert!(res.find("https://httpbin.org/get").is_some());
        });
    }

    #[test]
    fn test_kraken_client_error() {
        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            let c = KrakenClient {
                base_url: "http://foo.org".to_string(),
            };

            let query: HashMap<String, String> = HashMap::new();
            let res = c.req("get".to_string(), query).await;
            let expected = "error sending request for url (http://foo.org/get): error trying to connect: failed to lookup address information"; 
            assert!(res.is_err(), "should result in error if url does not exist");
            match res.err().unwrap() {
                Error::APIError(err) => assert!(err.find(expected) != None),
                _ => assert!(false),
            }
        });
    }
}
