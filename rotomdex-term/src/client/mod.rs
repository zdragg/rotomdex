mod rate_limiter;
mod retrier;

use std::{borrow::Cow, path::PathBuf};

use anyhow::Context;
use gix::protocol::transport::IsSpuriousError;
use http::StatusCode;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use rotomdex_core::{ApiError, CachedTransport, RelativePath};
use serde::Deserialize;

use crate::client::{rate_limiter::RateLimiter, retrier::Retrier};

#[derive(Clone, Debug)]
pub(crate) struct CachedClient {
    pub(crate) inner: ClientWithMiddleware,
}

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
    async fn read(&'_ self, path: &RelativePath) -> Result<Cow<'_, [u8]>, ApiError> {
        let base_url =
            url_macro::url!("https://raw.githubusercontent.com/zdragg/rotomdex-data/main/");
        let mut path = path.as_str().trim_matches('/').to_owned();
        if path.starts_with("api/") {
            let (endpoint, identifier) = path.rsplit_once('/').context("bad api path")?;
            if identifier.parse::<u64>().is_err() {
                let index_url = base_url
                    .join(&format!("{endpoint}/index.json"))
                    .context("invalid index url")?;
                let index: Index = self.inner.get(index_url).send().await
                    .context("error while fetching index")?
                    .error_for_status().context("index request failed")?
                    .json().await.context("invalid index")?;
                let entry = index.results.into_iter().find(|entry| entry.name == identifier)
                    .ok_or(ApiError::NotFound)?;
                path = entry.url.trim_matches('/').to_owned();
            }
            path.push_str("/index.json");
        }
        let full_url = base_url.join(&path).context("invalid url")?;
        let resp = self
            .inner
            .get(full_url)
            .send()
            .await
            .context("error while sending request")?;
        match resp.error_for_status() {
            Ok(resp) => Ok(Cow::Owned(
                resp.bytes()
                    .await
                    .context("error while trying to extract bytes from request")?
                    .to_vec(),
            )),
            Err(e) => {
                if e.status() == Some(StatusCode::NOT_FOUND) {
                    Err(ApiError::NotFound)
                } else {
                    Err(anyhow::Error::from(e).into())
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OfflineClient {
    base_path: PathBuf,
}

impl OfflineClient {
    pub fn new(resource_dir: PathBuf) -> Self {
        Self {
            base_path: resource_dir.to_owned(),
        }
    }
}

#[derive(Deserialize)]
struct Index {
    results: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    name: String,
    url: String,
}

impl OfflineClient {
    async fn create_valid_path(&self, relative_path: &RelativePath) -> anyhow::Result<PathBuf> {
        let relative_path = RelativePath::new(relative_path.as_str().trim_matches('/'));
        if !relative_path.as_str().starts_with("api/") {
            return Ok(relative_path.to_path(&self.base_path));
        }
        let (endpoint, identifier) = relative_path
            .as_str()
            .rsplit_once('/')
            .with_context(|| format!("bad api path: {}", relative_path))?;

        if identifier.parse::<u64>().is_ok() {
            return Ok(relative_path.to_path(&self.base_path).join("index.json"));
        }

        let index = tokio::fs::read(self.base_path.join(endpoint).join("index.json")).await?;

        let index: Index = serde_json::from_slice(&index)?;
        let entry = index
            .results
            .into_iter()
            .find(|entry| entry.name == identifier)
            .with_context(|| format!("pokémon \"{identifier}\" not found"))?; // This path can only be reached from Pokemon search for now

        Ok(self
            .base_path
            .join(entry.url.trim_matches('/'))
            .join("index.json"))
    }
}

#[async_trait::async_trait]
impl CachedTransport for OfflineClient {
    async fn read(&'_ self, path: &RelativePath) -> Result<Cow<'_, [u8]>, ApiError> {
        let file_path = self.create_valid_path(path).await?;
        let bytes = tokio::fs::read(file_path).await;
        match bytes {
            Ok(bytes) => Ok(bytes.into()),
            Err(e) => {
                if e.is_spurious() {
                    self.read(path).await
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    Err(ApiError::NotFound)
                } else {
                    Err(anyhow::Error::from(e).into())
                }
            }
        }
    }
}
