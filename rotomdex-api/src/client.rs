use core::fmt::Debug;

use alloc::{borrow::Cow, boxed::Box, rc::Rc};
pub use relative_path::{RelativePath, RelativePathBuf};
use serde::de::DeserializeOwned;

pub use crate::error::Error;

type Result<T> = core::result::Result<T, Error>;

#[async_trait::async_trait]
pub trait CachedTransport: Debug {
    async fn read(&'_ self, path: &RelativePath) -> Result<Cow<'_, [u8]>>;
}

#[derive(Debug)]
pub struct Client {
    pub transport: Rc<dyn CachedTransport>,
}

impl Client {
    pub fn new(transport: impl CachedTransport + 'static) -> Self {
        Self {
            transport: Rc::new(transport),
        }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &RelativePath) -> Result<T> {
        let body = self.get_raw(path).await?;
        Ok(serde_json::from_slice(body.as_ref())?)
    }

    pub async fn get_raw(&'_ self, path: &RelativePath) -> Result<Cow<'_, [u8]>> {
        let path = path.strip_prefix("/").unwrap_or(path);
        self.transport.read(path).await
    }
}
