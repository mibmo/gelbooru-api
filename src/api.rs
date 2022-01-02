use crate::{Client, Error};
use serde::Deserialize;
use hyper::body::Buf;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{AsRef, Into};

// marker trait for API types
trait ApiType: serde::de::DeserializeOwned {}

const API_BASE: &'static str = "https://gelbooru.com/index.php?page=dapi&q=index&json=1";

type QueryStrings<'a> = HashMap<&'a str, String>;

/*
use chrono::{offset::FixedOffset, DateTime};
#[derive(Clone, Copy, Debug)]
pub enum Ordering {
    Date,
    Count,
    Name,
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub id: u64,
    pub tag: String,
    pub count: u64,
    pub r#type: String,
    pub ambiguous: u64,
}

// @TODO: TagType enum
*/

#[derive(Deserialize, Debug)]
pub struct Post {
    pub source: String,
    pub directory: String,
    pub hash: String,
    pub height: u64,
    pub id: u64,
    pub image: String,
    pub change: u64,
    pub owner: String,
    pub parent_id: Option<u64>,
    pub rating: String,
    pub sample: u64,
    pub preview_height: u64,
    pub preview_width: u64,
    pub sample_height: u64,
    pub sample_width: u64,
    pub score: u64,
    pub tags: String,
    pub title: String,
    pub width: u64,
    pub file_url: String,
    pub created_at: String,
    pub post_locked: u64,
}

impl ApiType for Post {}

impl Post {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title<'a>(&'a self) -> &'a str {
        &self.title
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::offset::FixedOffset> {
        chrono::DateTime::parse_from_str(&self.created_at, "%a %b %d %H:%M:%S %z %Y")
            .expect("failed to parse DateTime")
    }

    pub fn rating<'a>(&'a self) -> crate::Rating {
        use crate::Rating::*;
        match &self.rating[0..1] {
            "s" => Safe,
            "q" => Questionable,
            "e" => Explicit,
            _ => unreachable!("non-standard rating"),
        }
    }

    pub fn owner<'a>(&'a self) -> &'a str {
        &self.owner
    }

    pub fn tags<'a>(&'a self) -> Vec<&'a str> {
        self.tags.split(' ').collect()
    }

    pub fn dimensions(&self) -> (u64, u64) {
        (self.width, self.height)
    }

    pub fn hash<'a>(&'a self) -> &'a str {
        &self.hash
    }

    pub fn image_url<'a>(&'a self) -> &'a str {
        &self.file_url
    }

    pub fn source<'a>(&'a self) -> &'a str {
        &self.source
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
    pub(crate) limit: Option<usize>,
    pub(crate) tags: Vec<Cow<'a, str>>,
    pub(crate) tags_raw: String,
    pub(crate) rating: Option<Rating>,
    pub(crate) sort_random: bool,
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

    /// Filter by content ratings.
    /// See [`Rating`](enum.rating.html).
    ///
    /// ## Example
    /// ```rust
    /// # async fn example() -> Result<(), ()> {
    /// # use gelbooru_api::{Client, Rating, posts};
    /// # let client = Client::public();
    /// posts()
    ///     .tags(&["hatsune_miku"])
    ///     .rating(Rating::Safe)
    ///     .send(&client)
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn rating(mut self, rating: Rating) -> Self {
        self.rating = Some(rating);
        self
    }

    /// Randomize the order of posts.
    ///
    /// This is a server-side meta-tag feature, and is only provided for completeness' sake.
    ///
    /// ## Example
    /// ```rust
    /// # async fn example() -> Result<(), ()> {
    /// # use gelbooru_api::{Client, Rating, posts};
    /// # let client = Client::public();
    /// posts()
    ///     .tags(&["hatsune_miku"])
    ///     .random(true)
    ///     .send(&client)
    ///     .await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
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

        let mut query_map: QueryStrings = Default::default();
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
