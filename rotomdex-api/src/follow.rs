use relative_path::RelativePath;
use serde::de::DeserializeOwned;

use crate::{
    client::Client,
    error::Error,
    model::resource::{ApiResource, NamedApiResource},
};

#[allow(async_fn_in_trait)]
pub trait Follow<T: DeserializeOwned> {
    async fn follow(&self, client: &Client) -> Result<T, Error>;
}

impl<T: DeserializeOwned> Follow<T> for NamedApiResource<T> {
    async fn follow(&self, client: &Client) -> Result<T, Error> {
        client.get(RelativePath::new(&self.url)).await
    }
}

impl<T: DeserializeOwned> Follow<T> for ApiResource<T> {
    async fn follow(&self, client: &Client) -> Result<T, Error> {
        client.get(RelativePath::new(&self.url)).await
    }
}
