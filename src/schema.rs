table! {
    trades (id) {
        id -> Int4,
        pair -> Varchar,
        price -> Float8,
        volume -> Float8,
        timestamp -> Int8,
    }
}
