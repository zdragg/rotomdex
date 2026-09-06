#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use anyhow::Context;
use async_trait::async_trait;
use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next};
use serde::Deserialize;

#[derive(Deserialize)]
struct Index {
    results: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    name: String,
    url: String,
}

/// What should be under path:
/// api/v2/pokemon-species/index.html
/// sprites/pokemon/132.png
pub(super) struct OfflineProvider {
    pub(super) path: PathBuf,
}

impl OfflineProvider {
    /// translates pokemon-species/rotom/ to pokemon-species/479/index.json
    async fn api_path(&self, request: &Request) -> anyhow::Result<PathBuf> {
        let relative = request.url().path().trim_matches('/');
        let (endpoint, identifier) = relative.rsplit_once('/').context("invalid API path")?;

        if identifier.parse::<u64>().is_ok() {
            return Ok(self.path.join(relative).join("index.json"));
        }

        let index = async_fs::read(self.path.join(endpoint).join("index.json")).await?;
        let index: Index = serde_json::from_slice(&index)?;
        let entry = index
            .results
            .into_iter()
            .find(|entry| entry.name == identifier)
            .with_context(|| format!("{identifier} not found in {endpoint}"))?;

        Ok(self.path.join(entry.url.trim_matches('/')).join("index.json"))
    }
}

#[async_trait]
impl Middleware for OfflineProvider {
    async fn handle(&self, req: Request, _extensions: &mut Extensions, _next: Next<'_>) -> Result<Response, Error> {
        let url = req.url().as_str();

        let path = if url.starts_with("https://pokeapi.co/") || url.starts_with("/") {
            Some(self.api_path(&req).await?)
        } else if let Some(rest) = url.strip_prefix("https://raw.githubusercontent.com/PokeAPI/sprites/master/") {
            Some(self.path.join(rest))
        } else if let Some(rest) = url.strip_prefix("https://raw.githubusercontent.com/PokeAPI/cries/main/") {
            Some(self.path.join(rest))
        } else {
            None
        }
        .context("invalid url, cannot map to offline path")?;

        let bytes = async_fs::read(&path).await.context("failed to read local file")?;

        let response = http::Response::builder()
            .status(http::StatusCode::OK)
            .body(bytes)
            .context("response cannot be built")?;

        Ok(response.into())
    }
}
