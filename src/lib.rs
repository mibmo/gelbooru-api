pub mod api;
pub mod auth;
pub mod client;

pub use api::*;
pub(crate) use auth::*;
pub use client::Client;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not parse date time")]
    DateTimeParse(#[from] chrono::format::ParseError),
    #[error("could not parse query string")]
    AuthParse(Option<std::num::ParseIntError>),
    #[error("a hyper-related error occured")]
    Hyper(#[from] hyper::Error),
    #[error("an error occured deserializing json response")]
    JsonDeserialize(#[from] serde_json::Error),
}
