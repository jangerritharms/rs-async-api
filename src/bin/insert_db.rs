use crytrade::repo::*;
use diesel::prelude::*;

fn main() {
    use crytrade::schema::trades;

    let connection = establish_connection();
    let new_trade = NewTradeEntity {
        symbol: String::from("ETHEUR"),
        price: 10.0,
        volume: 5.0,
        timestamp: 1000,
    };

    let trade: TradeEntity = diesel::insert_into(trades::table)
        .values(&new_trade)
        .get_result(&connection)
        .expect("Error saving new post");


    println!("Displaying {:?} trades", trade);
}
