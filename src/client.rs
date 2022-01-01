use crate::api::*;
use crate::Error;

use hyper::body::Buf;
use hyper_rustls::HttpsConnector;

type HClient = hyper::Client<HttpsConnector<hyper::client::HttpConnector>>;

const API_BASE: &'static str = "https://gelbooru.com";
const API_POSTS: &'static str = "/index.php?page=dapi&s=post&q=index&json=1";
const API_TAGS: &'static str = "/index.php?page=dapi&s=tag&q=index&json=1";
const API_COMMENTS: &'static str = "/index.php?page=dapi&s=comment&q=index&json=1";
const API_DELETED_IMAGES: &'static str = "/index.php?page=dapi&s=post&q=index&deleted=show&json=1";

pub struct Client {
    http_client: HClient,
}

impl Client {
    pub fn new() -> Self {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build();
        let http_client = hyper::Client::builder().build::<_, hyper::Body>(connector);

        Self { http_client }
    }

    pub async fn post_by_id(&self, id: u64) -> Result<Option<Post>, Error> {
        let url: hyper::Uri = format!("{}{}&id={id}", API_BASE, API_POSTS, id = id)
            .parse()
            .unwrap();

        self.send_req_parse_json::<Post>(url)
            .await
            .map(|posts| posts.into_iter().next())
    }

    pub async fn posts_new(&self, limit: u64) -> Result<Vec<Post>, Error> {
        let url: hyper::Uri = format!("{}{}&limit={limit}", API_BASE, API_POSTS, limit = limit)
            .parse()
            .unwrap();

        self.send_req_parse_json::<Post>(url).await
    }

    pub async fn posts_by_tag(&self, tags: &[&str], limit: u64) -> Result<Vec<Post>, Error> {
        let tags_query_string: String = tags.join("+");
        let url: hyper::Uri = format!(
            "{}{}&limit={limit}&tags={tags}",
            API_BASE,
            API_POSTS,
            limit = limit,
            tags = tags_query_string
        )
        .parse()
        .expect("failed to parse url");

        self.send_req_parse_json::<Post>(url).await
    }

    pub async fn tag_by_id(&self, id: u64) -> Result<Option<Tag>, Error> {
        let url: hyper::Uri = format!("{}{}&id={id}", API_BASE, API_TAGS, id = id)
            .parse()
            .unwrap();

        self.send_req_parse_json::<Tag>(url)
            .await
            .map(|posts| posts.into_iter().next())
    }

    pub async fn tag_by_name(&self, name: &str, limit: u64) -> Result<Option<Tag>, Error> {
        let url: hyper::Uri = format!(
            "{}{}&limit={limit}&name={name}",
            API_BASE,
            API_TAGS,
            limit = limit,
            name = name,
        )
        .parse()
        .expect("failed to parse url");

        self.send_req_parse_json::<Tag>(url)
            .await
            .map(|posts| posts.into_iter().next())
    }

    pub async fn tags_by_names(&self, names: &[&str], limit: u64, ordering: Ordering, ascending: bool) -> Result<Vec<Tag>, Error> {
        let order = order(ordering, ascending);
        let names_query_string: String = names.join("+");
        let url: hyper::Uri = format!(
            "{}{}{}&limit={limit}&names={names}",
            API_BASE,
            API_TAGS,
            order,
            limit = limit,
            names = names_query_string,
        )
        .parse()
        .expect("failed to parse url");

        self.send_req_parse_json::<Tag>(url).await
    }

    pub async fn tags_by_pattern(
        &self,
        pattern: &str,
        limit: u64,
        ordering: Ordering,
        ascending: bool,
    ) -> Result<Vec<Tag>, Error> {
        let order = order(ordering, ascending);
        let url: hyper::Uri = format!(
            "{}{}{}&limit={limit}&name_pattern={pattern}",
            API_BASE,
            API_TAGS,
            order,
            limit = limit,
            pattern = pattern,
        )
        .parse()
        .expect("failed to parse url");

        self.send_req_parse_json::<Tag>(url).await
    }

    async fn send_req_parse_json<T: serde::de::DeserializeOwned>(
        &self,
        url: hyper::Uri,
    ) -> Result<Vec<T>, Error> {
        let res = self
            .http_client
            .get(url)
            .await
            .map_err(|err| Error::Hyper(err))?;
        let body = hyper::body::aggregate(res)
            .await
            .map_err(|err| Error::Hyper(err))?;
        serde_json::from_reader(body.reader()).map_err(|err| Error::JsonDeserialize(err))
    }
}

fn order(ordering: Ordering, ascending: bool) -> String {
    format!(
        "&order={}&orderby={}",
        if ascending { "ASC" } else { "DESC" },
        format!("{:?}", ordering).to_lowercase(),
    )
}
