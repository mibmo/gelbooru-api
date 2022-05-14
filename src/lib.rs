#[cfg(test)]
mod test;

pub mod api;
mod auth;
mod client;
mod error;
pub use api::{Rating, Ordering, TagType};
//pub use api::{comments};
pub use auth::AuthDetails;
pub use client::Client;
pub use error::Error;

/// Gateway to interacting with the Posts API
///
/// ## Example
/// ```rust
/// # use gelbooru_api::{Client, Error, Rating, posts};
/// # async fn example() -> Result<(), Error> {
/// let client = Client::public();
///
/// posts()
///     .limit(50)                       // 50 posts
///     .rating(Rating::Safe)            // that have the safe rating
///     .tags(&["hatsune_miku", "solo"]) // and the `hatsune_miku` and `solo` tags
///     .send(&client)                   // send request
///     .await?;
///
/// # Ok(())
/// # }
/// ```
pub fn posts<'a>() -> api::PostsRequestBuilder<'a> {
    api::PostsRequestBuilder {
        limit: None, // server-side default is 100
        tags: Vec::new(),
        tags_raw: String::new(),
        rating: None,
        sort_random: false,
    }
}

/// Gateway to interacting with the Tags API
///
/// ## Example
/// ```rust
/// # use gelbooru_api::{Client, Error, Ordering, tags};
/// # async fn example() -> Result<(), Error> {
/// let client = Client::public();
///
/// tags()
///     .limit(5)
///     .pattern(&client, "_ol_")
///     .await?;
/// # Ok(())
/// # }
/// ```
pub fn tags() -> api::TagsRequestBuilder {
    api::TagsRequestBuilder::new()
}
