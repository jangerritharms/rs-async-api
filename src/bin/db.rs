use crytrade::repo;
use diesel::prelude::*;

fn main() {
    use crytrade::schema::trades::dsl::*;

    let connection = repo::establish_connection();
    let results = trades.filter(id.eq(1))
        .limit(5)
        .load::<repo::TradeEntity>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} trades", results.len());
    for trade in results {
        println!("{:?}", trade);
    }
}
