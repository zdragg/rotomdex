use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response, StatusCode};
use reqwest_middleware::{Error, Middleware, Next};

pub(super) struct Retrier {
    retry_count: usize,
}

impl Retrier {
    pub(super) fn new(retry_count: usize) -> Self {
        Self { retry_count }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Middleware for Retrier {
    async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> Result<Response, Error> {
        let mut retries = 0;
        loop {
            let request = req
                .try_clone()
                .ok_or_else(|| Error::middleware(std::io::Error::other("request cannot be cloned")))?;

            let result = next.clone().run(request, extensions).await;
            let not_found = match &result {
                Ok(response) => response.status() == StatusCode::NOT_FOUND,
                Err(_) => false,
            };

            if not_found || retries >= self.retry_count {
                return result;
            }
            retries += 1;
        }
    }
}
