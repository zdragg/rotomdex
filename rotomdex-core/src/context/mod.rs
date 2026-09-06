mod offline_provider;
mod rate_limiter;
mod retrier;

use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use reqwest::{Client, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustemon::client::{Environment, RustemonClient};

#[cfg(not(target_arch = "wasm32"))]
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};

use crate::{
    Version,
    context::{rate_limiter::RateLimiter, retrier::Retrier},
};

#[derive(Clone)]
pub(crate) struct ModelContext {
    pub(crate) pkmn_client: Arc<RustemonClient>,
    pub(crate) req_client: ClientWithMiddleware,
    pub(crate) version: Version,
}

impl ModelContext {
    fn base_builder() -> ClientBuilder {
        ClientBuilder::new(Client::new())
            .with(Retrier::new(5))
            .with(RateLimiter::new(64))
    }

    fn from_builder(builder: ClientBuilder) -> Self {
        let client = builder.build();

        let rustemon_wrapper = Arc::new(RustemonClient {
            base: Url::try_from(Environment::default()).unwrap(),
            client: client.clone(),
        });

        Self {
            pkmn_client: rustemon_wrapper,
            req_client: client,
            version: Version::default(),
        }
    }

    pub(super) fn new() -> Self {
        Self::from_builder(Self::base_builder())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn new_cache(cache_dir: PathBuf) -> Self {
        let cache_manager = http_cache_reqwest::CACacheManager::new(cache_dir, false);
        let cache = Cache(HttpCache {
            mode: CacheMode::Default,
            manager: cache_manager,
            options: HttpCacheOptions::default(),
        });

        Self::from_builder(Self::base_builder().with(cache))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// What should be under path:
    /// api/v2/pokemon-species/index.html
    /// sprites/pokemon/132.png
    pub(super) fn new_offline(path: PathBuf) -> Self {
        use crate::context::offline_provider::OfflineProvider;

        let offline = OfflineProvider { path };

        Self::from_builder(ClientBuilder::new(Client::new()).with(offline))
    }
}
