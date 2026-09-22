use std::{borrow::Cow, path::PathBuf};

use relative_path::RelativePath;
use rotomdex_api::{CachedTransport, NotFoundSnafu, OtherTransportError, TransportError};
use snafu::Snafu;

#[derive(Clone, Debug)]
pub(crate) struct OfflineClient {
    base_path: PathBuf,
}

#[derive(Debug, Snafu)]
enum OfflineFetchError {
    #[snafu(display("Filesystem error: {inner}"))]
    Fs { inner: std::io::Error },
}

impl OtherTransportError for OfflineFetchError {}

impl OfflineClient {
    pub fn new(resource_dir: PathBuf) -> Self {
        Self {
            base_path: resource_dir.to_owned(),
        }
    }
}

#[async_trait::async_trait]
impl CachedTransport for OfflineClient {
    async fn read(&'_ self, path: &RelativePath) -> Result<Cow<'_, [u8]>, TransportError> {
        let full_path = path.to_path(&self.base_path);
        let bytes = tokio::fs::read(full_path).await;

        match bytes {
            Ok(bytes) => Ok(Cow::Owned(bytes)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    NotFoundSnafu { path }.fail()
                } else {
                    FsSnafu { inner: e }.fail()?
                }
            }
        }
    }
}
