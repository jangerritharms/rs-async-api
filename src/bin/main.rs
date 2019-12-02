#![feature(stmt_expr_attributes, proc_macro_hygiene)]
use crytrade::api;
use crytrade::kraken;
use crytrade::repo::*;
use crytrade::schema::trades;
use diesel::prelude::*;
use dotenv::dotenv;
use futures::stream::StreamExt;
use std::env;

struct Config {
    database_url: String,
}

fn load_config() -> Result<Config, std::env::VarError> {
    dotenv().ok();

    Ok(Config {
        database_url: env::var("DATABASE_URL")?,
    })
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = match load_config() {
        Ok(config) => {
            println!("Loaded configuration");
            config
        }
        Err(err) => {
            println!("{:?}", err);
            std::process::exit(1)
        }
    };

    let conn = match establish_connection(config.database_url) {
        Ok(conn) => {
            println!("Connected to db");
            conn
        }
        Err(err) => {
            println!("Couldn't connect to db");
            println!("{}", err);
            std::process::exit(2);
        }
    };

    let client = api::KrakenClient {
        base_url: "https://api.kraken.com/0/public".to_string(),
    };
    let kraken = kraken::Kraken { client };
    let stream = kraken.history_since_until_now("ETHEUR".to_string(), 1575100000000000000);
    let trades = stream
        .then(|trade| {
            async {
                let new_trade: NewTradeEntity = trade.into();
                diesel::insert_into(trades::table)
                    .values(&new_trade)
                    .get_result(&conn)
                    .expect("Error saving new post")
            }
        })
        .collect::<Vec<TradeEntity>>()
        .await;

    println!("Saved {:?} ", trades.len());

    Ok(())
}
