mod rate_limiter;
mod retrier;

use std::{borrow::Cow, path::PathBuf};

use http::StatusCode;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use relative_path::{RelativePath, RelativePathBuf};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rotomdex_api::{CachedTransport, NotFoundSnafu, OtherTransportError, TransportError};
use snafu::{ResultExt, Snafu, ensure};

use rate_limiter::RateLimiter;
use retrier::Retrier;
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct CachedClient {
    pub(crate) inner: ClientWithMiddleware,
}

#[derive(Debug, Snafu)]
enum OnlineFetchError {
    #[snafu(display("{path} cannot be attached to {base_url}"))]
    JoinUrl {
        source: url::ParseError,
        base_url: Url,
        path: RelativePathBuf,
    },
    #[snafu(display("network request error: {inner}"))]
    ReqwestMiddleware { inner: reqwest_middleware::Error },
    #[snafu(display("network request error: {inner}"))]
    Reqwest { inner: reqwest::Error },
    #[snafu(display("Status code {status_code} is unexpected"))]
    InvalidStatusCode { status_code: StatusCode },
}

impl OtherTransportError for OnlineFetchError {}

impl CachedClient {
    pub fn new(cache_dir: PathBuf) -> Self {
        let cache_manager = http_cache_reqwest::CACacheManager::new(cache_dir, false);
        let cache = Cache(HttpCache {
            mode: CacheMode::Default,
            manager: cache_manager,
            options: HttpCacheOptions::default(),
        });

        let client = ClientBuilder::new(reqwest::Client::new())
            .with(Retrier::new(5))
            .with(RateLimiter::new(32))
            .with(cache)
            .build();

        Self { inner: client }
    }
}

#[async_trait::async_trait]
impl CachedTransport for CachedClient {
    async fn read(&'_ self, path: &RelativePath) -> Result<Cow<'_, [u8]>, TransportError> {
        let base_url =
            url_macro::url!("https://raw.githubusercontent.com/zdragg/rotomdex-data/main/");

        let full_url = base_url
            .join(path.as_str())
            .context(JoinUrlSnafu { base_url, path })?;

        let resp = match self.inner.get(full_url).send().await {
            Ok(resp) => {
                // Only status code 200 should be considered a success
                let status_code = resp.status();
                ensure!(status_code != StatusCode::NOT_FOUND, NotFoundSnafu { path });
                ensure!(
                    status_code == StatusCode::OK,
                    InvalidStatusCodeSnafu { status_code }
                );
                resp
            }
            Err(err) => ReqwestMiddlewareSnafu { inner: err }.fail()?,
        };

        let bytes = match resp.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => ReqwestSnafu { inner: err }.fail()?,
        };

        Ok(Cow::Owned(bytes.to_vec()))
    }
}
