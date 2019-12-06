use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::ConnectionError;

use crate::trade::Trade;
use crate::schema::trades;

pub fn establish_connection(database_url: String) -> Result<PgConnection, ConnectionError>  {
    Ok(PgConnection::establish(&database_url)?)
}

#[derive(Insertable)]
#[table_name="trades"]
pub struct NewTradeEntity {
    pub pair: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: i64,
}

#[derive(Queryable, Debug)]
pub struct TradeEntity {
    pub id: i32,
    pub pair: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: i64,
}

impl From<Trade> for NewTradeEntity {
    fn from(trade: Trade) -> Self {
        NewTradeEntity {
            pair: trade.pair,
            price: trade.price,
            volume: trade.volume,
            timestamp: trade.timestamp as i64,
        }
    }
}

