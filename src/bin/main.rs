#![feature(stmt_expr_attributes, proc_macro_hygiene)]
use crytrade::api;
use crytrade::kraken;
use crytrade::repo::*;
use crytrade::schema::trades;
use diesel::prelude::*;
use dotenv::dotenv;
use futures::stream::StreamExt;
use std::env;
use clap::{App, Arg};
use log::{debug, info, error, Level};

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
    simple_logger::init_with_level(Level::Info).unwrap();

    let matches = App::new("kraken-sync")
        .version("1.0")
        .about("Sync kraken trades to a database")
        .author("Jan-Gerrit Harms")
        .arg(Arg::with_name("pair")
             .short("p")
             .long("pair")
             .value_name("PAIR")
             .required(true)
             .help("Pair for which trades should be synced")
             .takes_value(true))
        .arg(Arg::with_name("since")
             .short("s")
             .long("since")
             .value_name("TIMESTAMP")
             .required(true)
             .help("Timestamp from which to start trade sync")
             .takes_value(true))
        .get_matches();

    let config = match load_config() {
        Ok(config) => {
            info!("Loaded configuration");
            config
        }
        Err(err) => {
            error!("{:?}", err);
            std::process::exit(1)
        }
    };

    let conn = match establish_connection(config.database_url) {
        Ok(conn) => {
            info!("Connected to db");
            conn
        }
        Err(err) => {
            error!("Couldn't connect to db: {:?}", err);
            std::process::exit(2);
        }
    };

    let client = api::KrakenClient {
        base_url: "https://api.kraken.com/0/public".to_string(),
    };
    let kraken = kraken::Kraken { client };

    let since = matches.value_of("since").unwrap().parse::<u64>().unwrap();
    let pair = matches.value_of("pair").unwrap().to_string();

    let stream = kraken.history_since_until_now(pair, since);
    let trades = stream
        .map(|trade| {
            debug!("Received Trade: {:?}", trade);
            trade.into()
        })
        .collect::<Vec<NewTradeEntity>>()
        .await;

    info!("Retrieved {:?} ", trades.len());

    diesel::insert_into(trades::table)
        .values(&trades)
        .execute(&conn)
        .expect("Error saving new posts");

    Ok(())
}
