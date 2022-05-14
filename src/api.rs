//! API types and methods
//!
//! Use the associated functions at the root module and `RequestBuilder`s to send requests.

use crate::{Client, Error};
use hyper::body::Buf;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{AsRef, Into};

// marker trait for API types
trait ApiQuery: serde::de::DeserializeOwned {}

const API_BASE: &'static str = "https://gelbooru.com/index.php?page=dapi&q=index&json=1";

type QueryStrings<'a> = HashMap<&'a str, String>;

#[derive(Deserialize, Debug)]
pub struct Attributes {
    pub limit: usize,
    pub offset: usize,
    pub count: usize,
}

#[derive(Deserialize, Debug)]
pub struct PostQuery {
    #[serde(rename = "@attributes")]
    pub attributes: Attributes,
    #[serde(rename = "post", default = "Vec::new")]
    pub posts: Vec<Post>,
}

#[derive(Deserialize, Debug)]
pub struct TagQuery {
    #[serde(rename = "@attributes")]
    pub attributes: Attributes,
    #[serde(rename = "tag", default = "Vec::new")]
    pub tags: Vec<Tag>,
}

/// Post on Gelbooru
#[derive(Deserialize, Debug)]
pub struct Post {
    pub source: String,
    pub directory: String,
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

impl ApiQuery for PostQuery {}

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

    pub fn rating<'a>(&'a self) -> Rating {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    /// # use gelbooru_api::{Client, Error, posts};
    /// # async fn example() -> Result<(), Error> {
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
    /// # use gelbooru_api::{Client, Error, posts};
    /// # async fn example() -> Result<(), Error> {
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
    /// # use gelbooru_api::{Client, Error, posts};
    /// # async fn example() -> Result<(), Error> {
    /// # let client = Client::public();
    /// posts()
    ///     .tags(&["herro", "world"])
    ///     .clear_tags() // wait, nevermind! clear tags.
    ///     .tags(&["hello", "world"])
    ///     .send(&client)
    ///     .await?;
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
    /// # use gelbooru_api::{Client, Error, Rating, posts};
    /// # async fn example() -> Result<(), Error> {
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
    /// # use gelbooru_api::{Client, Error, Rating, posts};
    /// # async fn example() -> Result<(), Error> {
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

    pub async fn send(self, client: &Client) -> Result<PostQuery, Error> {
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

        let mut qs: QueryStrings = Default::default();
        qs.insert("s", "post".to_string());
        qs.insert("limit", self.limit.unwrap_or(100).to_string());
        qs.insert("tags", tags);

        query_api(client, qs).await
    }
}

/// Tag on Gelbooru
#[derive(Deserialize, Debug)]
pub struct Tag {
    pub id: String,
    pub tag: String,
    pub count: String,
    pub r#type: String,
    pub ambiguous: String,
}

impl ApiQuery for TagQuery {}

impl Tag {
    pub fn id(&self) -> u64 {
        self.id.parse().expect("tag's ID not a number")
    }

    pub fn tag<'a>(&'a self) -> &'a str {
        &self.tag
    }

    pub fn count(&self) -> u64 {
        self.count.parse().expect("tag's count not a number")
    }

    pub fn tag_type(&self) -> TagType {
        use TagType::*;
        match self.r#type.as_str() {
            "artist" => Artist,
            "character" => Character,
            "copyright" => Copyright,
            "deprecated" => Deprecated,
            "metadata" => Metadata,
            "tag" => Tag,
            _ => unreachable!("non-standard tag type"),
        }
    }

    pub fn ambigious(&self) -> bool {
        if self.ambiguous == "0" {
            false
        } else {
            true
        }
    }
}

/// The type of a tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagType {
    Artist,
    Character,
    Copyright,
    Deprecated,
    Metadata,
    Tag,
}

/// Determines what field sorts tags in a query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ordering {
    Date,
    Count,
    Name,
}

/// Request builder for the Tags endpoint.
///
/// See the [`tags`](fn.tags.html) function for proper usage.
#[derive(Clone, Debug)]
pub struct TagsRequestBuilder {
    limit: Option<usize>,
    after_id: Option<usize>,
    order_by: Option<Ordering>,
    ascending: Option<bool>,
}

enum TagSearch<'a> {
    Name(&'a str),
    Names(Vec<&'a str>),
    Pattern(&'a str),
}

impl TagsRequestBuilder {
    pub(crate) fn new() -> Self {
        Self {
            limit: None,
            after_id: None,
            order_by: None,
            ascending: None,
        }
    }

    /// Amount of tags to recieve.
    ///
    /// When unspecified, default limit is 100, as set by the server.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn after_id(mut self, id: usize) -> Self {
        self.after_id = Some(id);
        self
    }

    pub fn ascending(mut self, ascending: bool) -> Self {
        self.ascending = Some(ascending);
        self
    }

    /// How tags are sorted.
    /// _Date, Count, Name._
    ///
    /// This is mainly useful with [`send`](#method.send), but ordering works with the other search
    /// methods as well ([`name`](#method.name), [`names`](#method.names), [`pattern`](#method.pattern))
    ///
    /// ## Example
    /// ```rust
    /// # use gelbooru_api::{Client, Error, Ordering, tags};
    /// # async fn example() -> Result<(), Error> {
    /// # let client = Client::public();
    /// tags()
    ///     .limit(5)                 // 5 tags
    ///     .ascending(true)             // descending
    ///     .order_by(Ordering::Date) // according to creation-time
    ///     .send(&client)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn order_by(mut self, ordering: Ordering) -> Self {
        self.order_by = Some(ordering);
        self
    }

    /// Query for tags without name/pattern specifier.
    ///
    /// ## Example
    /// ```rust
    /// # use gelbooru_api::{Client, Error, Ordering, tags};
    /// # async fn example() -> Result<(), Error> {
    /// # let client = Client::public();
    /// tags()
    ///     .limit(10)                 // 10 tags
    ///     .ascending(false)             // descending
    ///     .order_by(Ordering::Count) // according to count (how many posts have tag)
    ///     .send(&client)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(self, client: &Client) -> Result<TagQuery, Error> {
        self.search(client, None).await
    }

    /// Pull data for given tag
    ///
    /// ## Example
    /// ```rust
    /// # use gelbooru_api::{Client, Error, Ordering, tags};
    /// # async fn example() -> Result<(), Error> {
    /// # let client = Client::public();
    /// tags()
    ///     .name(&client, "solo")
    ///     .await?;
    /// # Ok(())
    /// # }
    pub async fn name<S: AsRef<str>>(self, client: &Client, name: S) -> Result<Option<Tag>, Error> {
        let search = TagSearch::Name(name.as_ref());
        self.search(client, Some(search))
            .await
            .map(|tags| tags.tags.into_iter().next())
    }

    /// Pull data for the specified tags
    ///
    /// Tag limit is automatically set to accompany all the names.
    ///
    /// ## Example
    /// ```rust
    /// # use gelbooru_api::{Client, Error, Ordering, tags};
    /// # async fn example() -> Result<(), Error> {
    /// # let client = Client::public();
    /// tags()
    ///     .names(&client, &["solo", "hatsune_miku"])
    ///     .await?;
    /// # Ok(())
    /// # }
    pub async fn names<S: AsRef<str>>(
        self,
        client: &Client,
        names: &[S],
    ) -> Result<TagQuery, Error> {
        let names = names.iter().map(|name| name.as_ref()).collect();
        let search = TagSearch::Names(names);
        self.search(client, Some(search)).await
    }

    /// Search for tags with a pattern.
    ///
    /// Use `_` for single-character wildcards and `%` for multi-character wildcards.
    /// (`%choolgirl%` would act as `*choolgirl*` wildcard search)
    ///
    /// ## Example
    /// ```rust
    /// # use gelbooru_api::{Client, Error, Ordering, tags};
    /// # async fn example() -> Result<(), Error> {
    /// # let client = Client::public();
    /// tags()
    ///     .limit(10)
    ///     .pattern(&client, "%o_o") // matches regex /.*o.o/
    ///     .await?;
    /// # Ok(())
    /// # }
    pub async fn pattern<S: AsRef<str>>(
        self,
        client: &Client,
        pattern: S,
    ) -> Result<TagQuery, Error> {
        let search = TagSearch::Pattern(pattern.as_ref());
        self.search(client, Some(search)).await
    }

    async fn search(
        self,
        client: &Client,
        search: Option<TagSearch<'_>>,
    ) -> Result<TagQuery, Error> {
        let limit = self.limit.unwrap_or_else(|| {
            use TagSearch::*;
            match &search {
                Some(Name(_)) => 1,
                Some(Names(names)) => names.len(),
                _ => 100,
            }
        });

        let mut qs: QueryStrings = Default::default();
        qs.insert("s", "tag".to_string());
        qs.insert("limit", limit.to_string());

        if let Some(id) = self.after_id {
            qs.insert("after_id", id.to_string());
        }

        if let Some(ordering) = self.order_by {
            use Ordering::*;
            let order_by = match ordering {
                Date => "date",
                Count => "count",
                Name => "name",
            }
            .to_string();
            qs.insert("orderby", order_by);
        }

        if let Some(ascending) = self.ascending {
            qs.insert("order", if ascending { "ASC" } else { "DESC" }.to_string());
        }

        if let Some(search) = search {
            use TagSearch::*;
            let (mode, mode_value) = match search {
                Name(name) => ("name", name.to_string()),
                Names(names) => ("names", names.join("+")),
                Pattern(pattern) => ("name_pattern", pattern.to_string()),
            };
            qs.insert(mode, mode_value);
        }

        query_api(client, qs).await
    }
}

/*
 * @TODO: add support for reading XML, since Comments & Deleted Images APIs don't support
 * outputting in json.

#[derive(Deserialize, Debug)]
pub struct Comment {}

impl ApiType for Comment {}

pub async fn comments(client: &Client, post_id: u64) -> Result<Vec<Comment>, Error> {
        let mut qs: QueryStrings = Default::default();
        qs.insert("s", "comment".to_string());
        qs.insert("post_id", post_id.to_string());

        query_api(client, qs).await
}
*/

// internal function as to DRY
async fn query_api<T: ApiQuery>(client: &Client, mut qs: QueryStrings<'_>) -> Result<T, Error> {
    if let Some(auth) = &client.auth {
        qs.insert("user_id", auth.user.to_string());
        qs.insert("api_key", auth.key.clone());
    }

    let query_string: String = qs
        .iter()
        .map(|(query, value)| format!("&{}={}", query, value))
        .collect();

    let uri = format!("{}{}", API_BASE, query_string)
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

    serde_json::from_reader(body.reader()).map_err(|err| Error::JsonDeserialize(err))
}
