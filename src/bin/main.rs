#![feature(stmt_expr_attributes, proc_macro_hygiene)]
use crytrade::kraken;
use crytrade::trade::{Trade, TradeSymbol};
use crytrade::repo::*;
use crytrade::trade::{TradeAPI};
use std::env;
use dotenv::dotenv;
use diesel::prelude::*;
// use futures::stream::{self, StreamExt};
// use futures::{future, Future, Stream};
// use futures::{stream, Stream};
use futures::stream::{StreamExt};

struct Config {
    database_url: String,
}

fn load_config() -> Result<Config, std::env::VarError> {
    dotenv().ok();

    Ok(Config {
        database_url: env::var("DATABASE_URL")?
    })
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use crytrade::schema::trades;

    let config = match load_config() {
        Ok(config) => {
            println!("Loaded configuration");
            config
        },
        Err(err) => {
            println!("{:?}", err);
            std::process::exit(1)
        }
    };

    // let conn = match establish_connection(config.database_url) {
    //     Ok(conn) => {
    //         println!("Connected to db");
    //         conn
    //     },
    //     Err(err) => {
    //         println!("Couldn't connect to db");
    //         println!("{}", err);
    //         std::process::exit(2);
    //     }
    // };

    let kraken = kraken::Kraken {
        base_url: "https://api.kraken.com/0/public".to_string(),
    };
    let sym = TradeSymbol {
        base_currency: "ETH".to_string(),
        quote_currency: "EUR".to_string(),
    };
    
    let stream = kraken.history_since_until_now(sym, 1573838847178106200);
    let trades = stream.collect::<Vec<kraken::KrakenTrade>>().await;
    println!("Retrieved {} trades", trades.len());

    Ok(())
}
