use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse authentication query string")]
    ParseAuth,
    #[error("could not parse user id")]
    ParseUserId(std::num::ParseIntError),
    #[error("request error")]
    Request(#[from] hyper::Error),
    #[error("an error occured deserializing json response")]
    JsonDeserialize(#[from] serde_json::Error),
    #[error("could not parse request Uri")]
    UriParse(#[from] http::uri::InvalidUri),
}
