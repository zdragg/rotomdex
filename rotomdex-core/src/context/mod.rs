mod rate_limiter;

use std::{path::PathBuf, sync::Arc};

use reqwest::{Client, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rustemon::client::{Environment, RustemonClient};

#[cfg(not(target_arch = "wasm32"))]
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};

use crate::{Version, context::rate_limiter::RateLimiter};

#[derive(Clone)]
pub(crate) struct ModelContext {
    pub(crate) pkmn_client: Arc<RustemonClient>,
    pub(crate) req_client: ClientWithMiddleware,
    pub(crate) version: Version,
}

impl ModelContext {
    pub(super) fn new(cache_dir: Option<PathBuf>) -> Self {
        let builder = ClientBuilder::new(Client::new());

        // 16 concurrent requests at a time
        let builder = builder.with(RateLimiter::new(16));

        // Add cache middleware on the outermost layer if needed
        // Parity with the browser cache behavior on wasm (where it is cached on the innermost layer) is achieved
        #[cfg(not(target_arch = "wasm32"))]
        let builder = if let Some(cache_dir) = cache_dir {
            let cache_manager = http_cache_reqwest::CACacheManager::new(cache_dir, false);
            let cache_middleware = Cache(HttpCache {
                mode: CacheMode::Default,
                manager: cache_manager,
                options: HttpCacheOptions::default(),
            });
            builder.with(cache_middleware)
        } else {
            builder
        };

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
}
