#![feature(try_trait)]
#[macro_use]
extern crate diesel;

pub mod kraken;
pub mod trade;
pub mod schema;
pub mod repo;
pub mod api;
pub mod error;

