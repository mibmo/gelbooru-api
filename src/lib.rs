#[cfg(tests)]
mod test;

pub mod api;
pub use api::*;

use hyper::body::Buf;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{AsRef, Into};

const API_BASE: &'static str = "https://gelbooru.com/index.php?page=dapi&q=index&json=1&s=";

type HClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

#[derive(Clone, Debug)]
pub struct AuthDetails {
    user: usize,
    key: String,
}

impl AuthDetails {
    pub fn from_query_string(qs: &str) -> Result<Self, Error> {
        let user_start = qs.find("&user_id=").ok_or(Error::ParseAuth)?;
        let user_raw = &qs[user_start + 9..];
        let user = user_raw
            .parse()
            .map_err(|err| Error::ParseUserId(err))?;
        let key = qs[9..user_start].to_string();

        Ok(Self { user, key })
    }
}

pub struct Client {
    http_client: HClient,
    auth: Option<AuthDetails>,
}

impl Client {
    fn base() -> Self {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build();
        let http_client = hyper::Client::builder().build::<_, hyper::Body>(connector);

        Self {
            http_client,
            auth: None,
        }
    }

    pub fn public() -> Self {
        Self::base()
    }

    pub fn with_auth(details: AuthDetails) -> Self {
        let mut client = Self::base();
        client.auth = Some(details);
        client
    }
}

/// The content rating of a post.
///
/// See [this forum post](https://gelbooru.com/index.php?page=wiki&s=view&id=2535) for an in-depth explanation of the 3 ratings.
#[derive(Clone, Debug)]
pub enum Rating {
    Safe,
    Questionable,
    Explicit,
}

/// Request builder for the Posts endpoint.
///
/// See the [`posts`](fn.posts.html) function for proper usage.
#[derive(Clone, Debug)]
pub struct PostsRequestBuilder<'a> {
    limit: Option<usize>,
    tags: Vec<Cow<'a, str>>,
    tags_raw: String,
    rating: Option<Rating>,
    sort_random: bool,
}

impl<'a> PostsRequestBuilder<'a> {
    /// Amount of posts to recieve.
    ///
    /// When unspecified, default limit is 100, as set by the server.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add a single tag to the list of tags to search for.
    /// To clear already set tags, see [`clear_tags`](#method.clear_tags).
    ///
    /// ## Example
    /// ```rust
    /// # async fn example() -> Result<(), ()> {
    /// # use gelbooru_api::{Client, posts};
    /// # let client = Client::public();
    /// posts()
    ///     .tag("hello")
    ///     .tag("world".to_string())
    ///     .send(&client)
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn tag<S: Into<Cow<'a, str>>>(mut self, tag: S) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Tags to search for.
    /// Any tag combination that works on the website will work here, including meta-tags.
    ///
    /// Can be chained; previously added tags are not overridden.
    /// To clear already set tags, see [`clear_tags`](#method.clear_tags).
    ///
    /// ## Example
    /// ```rust
    /// # async fn example() -> Result<(), ()> {
    /// # use gelbooru_api::{Client, posts};
    /// # let client = Client::public();
    /// posts()
    ///     .tags(&["hello", "world"])
    ///     .tags(&vec!["how", "are", "you?"])
    ///     .send(&client)
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn tags<S: AsRef<str>>(mut self, tags: &'a [S]) -> Self {
        let mut other = tags
            .iter()
            .map(|s| Cow::from(s.as_ref()))
            .collect::<Vec<_>>();
        self.tags.append(&mut other);
        self
    }

    /// Append string directly to tag search
    ///
    /// !! These are not checked when being submitted !!
    /// You can easily mess up the query construction using these.
    ///
    /// Probably only useful for setting meta-tags.
    pub fn tags_raw<S: std::string::ToString>(mut self, raw_tags: S) -> Self {
        self.tags_raw = raw_tags.to_string();
        self
    }

    /// Clears the list of tags to search for.
    /// Tags set using [`tags_raw`](#method.tags_raw) are also cleared.
    ///
    ///
    /// ## Example
    /// ```rust
    /// # async fn example() -> Result<(), ()> {
    /// # use gelbooru_api::{Client, posts};
    /// # let client = Client::public();
    /// posts()
    ///     .tags(&["herro", "world"])
    ///     .clear_tags() // wait, nevermind! clear tags.
    ///     .tags(&["hello", "world"])
    ///     .send(&client)
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_tags(mut self) -> Self {
        self.tags = Vec::new();
        self.tags_raw = String::new();
        self
    }

    /// Filter by ratings
    pub fn rating(mut self, rating: Rating) -> Self {
        self.rating = Some(rating);
        self
    }

    /// Randomize the order of posts
    pub fn random(mut self, random: bool) -> Self {
        self.sort_random = random;
        self
    }

    pub async fn send(self, client: &Client) -> Result<Vec<Post>, Error> {
        let mut tags = String::new();
        if let Some(rating) = self.rating {
            tags.push_str(&format!("rating:{:?}+", rating).to_lowercase());
        }
        if self.sort_random {
            tags.push_str("sort:random+");
        }
        tags.push_str(&self.tags.join("+"));
        if !self.tags_raw.is_empty() {
            tags.push('+');
            tags.push_str(&self.tags_raw);
        }

        let mut query_map: HashMap<&str, _> = HashMap::new();
        query_map.insert("limit", self.limit.unwrap_or(100).to_string());
        query_map.insert("tags", tags);

        if let Some(auth) = &client.auth {
            query_map.insert("user_id", auth.user.to_string());
            query_map.insert("api_key", auth.key.clone());
        }

        let query_string: String = query_map
            .iter()
            .map(|(query, value)| format!("{}={}&", query, value))
            .collect();

        // error: Error::UriParse(err)
        let uri = format!("{}post&{}", API_BASE, query_string)
            .parse::<hyper::Uri>()
            .map_err(|err| Error::UriParse(err))?;

        let res = client
            .http_client
            .get(uri)
            .await
            .map_err(|err| Error::Request(err))?;
        let body = hyper::body::aggregate(res)
            .await
            .map_err(|err| Error::Request(err))?;
        let posts: Vec<Post> =
            serde_json::from_reader(body.reader()).map_err(|err| Error::JsonDeserialize(err))?;

        Ok(posts)
    }
}

/// Gateway to the Posts api
///
/// ## Example
/// ```rust
/// # async fn example() -> Result<(), ()> {
/// # use gelbooru_api::{Client, Rating, posts};
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
pub fn posts<'a>() -> PostsRequestBuilder<'a> {
    PostsRequestBuilder {
        limit: None, // server-side default is 100
        tags: Vec::new(),
        tags_raw: String::new(),
        rating: None,
        sort_random: false,
    }
}

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
