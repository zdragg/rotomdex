use async_lock::Semaphore;
use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next};

pub(crate) struct RateLimiter {
    semaphore: Semaphore,
}

impl RateLimiter {
    pub(crate) fn new(n: usize) -> Self {
        Self {
            semaphore: Semaphore::new(n),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Middleware for RateLimiter {
    async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> Result<Response, Error> {
        let _guard = self.semaphore.acquire().await;
        next.run(req, extensions).await
    }
}
