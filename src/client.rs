use crate::AuthDetails;

type HClient = hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;

/// Gelbooru API client.
/// Used for authentication requests.
///
/// Should generally be reused for multiple requests.
pub struct Client {
    pub(crate) http_client: HClient,
    pub(crate) auth: Option<AuthDetails>,
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

    /// A basic unauthenticated client.
    ///
    /// May incur rate-limiting.
    pub fn public() -> Self {
        Self::base()
    }

    /// An authenticated client.
    ///
    /// May incur rate-limiting in extreme cases.
    /// Users that have supported on Patreon have no rate-limiting whatsoever.
    pub fn with_auth(details: AuthDetails) -> Self {
        let mut client = Self::base();
        client.auth = Some(details);
        client
    }
}
