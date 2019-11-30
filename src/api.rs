use async_trait::async_trait;
use serde::Serialize;
use std::error::Error;


pub struct KrakenClient {
    pub base_url: String,
}

#[async_trait]
pub trait HTTPRequest {
    async fn req<Req>(
        &self,
        endpoint: String,
        query: Req,
    )  -> std::result::Result<String, Box<dyn Error>>
        where
        Req: Serialize + Sized + std::marker::Sync + std::marker::Send;
}


#[async_trait]
impl HTTPRequest for KrakenClient {
    async fn req<Req>(
        &self,
        endpoint: String,
        query: Req,
    ) -> std::result::Result<String, Box<dyn Error>>
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

    #[test]
    fn test_kraken_client_req() {
        let mut rt = Runtime::new().unwrap();

        rt.block_on(async {
            let c = KrakenClient {
                base_url: "http://httpbin.org".to_string(),
            };

            let query: HashMap<String, String> = HashMap::new();
            let res: String = c.req("get".to_string(), query).await.unwrap();
            println!("{}", res);
            res.find("https://httpbin.org/get").unwrap();
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
            assert!(res.is_err(), "should result in error if url does not exist")
        });
    }
}
