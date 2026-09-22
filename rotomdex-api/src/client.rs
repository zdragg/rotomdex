use core::fmt::Debug;

use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    rc::Rc,
    string::String,
    vec::Vec,
};
pub use relative_path::RelativePath;
use relative_path::RelativePathBuf;
use serde::de::DeserializeOwned;
use snafu::OptionExt;

use crate::{
    BadPathSnafu, NotFoundInIndexSnafu,
    error::{Error, TransportError},
};

/// See [rotomdex-data](https://github.com/zdragg/rotomdex-data)
/// for data expected to be found under the relative path.
#[async_trait::async_trait]
pub trait CachedTransport: Debug {
    async fn read(&'_ self, path: &RelativePath) -> Result<Cow<'_, [u8]>, TransportError>;
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

    /// Fetches a resource under `path` using the provided transport, and serializes into `T`.
    pub async fn get<T, P>(&self, path: P) -> Result<T, Error>
    where
        T: DeserializeOwned,
        P: AsRef<RelativePath>,
    {
        let path = path.as_ref();
        let body = self.get_bytes(&path).await?;
        Self::serialize(body).await
    }

    /// Fetches a resource under `path` using the provided transport, and returns raw bytes.
    pub async fn get_bytes<P>(&'_ self, path: P) -> Result<Cow<'_, [u8]>, Error>
    where
        P: AsRef<RelativePath>,
    {
        let path = self.normalize(path).await?;
        Ok(self.transport.read(&path).await?)
    }

    async fn serialize<T>(body: Cow<'_, [u8]>) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        Ok(serde_json::from_slice(body.as_ref())?)
    }

    /// Turns a resource path into a proper relative path that points to a file.\
    /// e.g. `/api/v2/pokemon-species/pikachu/` -> `api/v2/pokemon-species/25/index.json`
    async fn normalize<P>(&self, path: P) -> Result<RelativePathBuf, Error>
    where
        P: AsRef<RelativePath>,
    {
        let path = path.as_ref().as_str();

        // Turn path into a relative path
        let path = path.strip_prefix("/").unwrap_or(path);

        let path = if path.ends_with("/") {
            // Path points to a folder. There are two cases:
            // 1. The folder is a number, e.g. `api/v2/pokemon/25`
            // We can append `index.json` to the end to get the actual file.
            //
            // 2. The folder is NOT a number, e.g.`api/v2/pokemon/pikachu`
            // To find the corresponding number, we need to get `api/v2/pokemon/index.json`,
            // parse it to [`Index`], and find the corresponding entry. It will contain
            // the path to the actual file we need, e.g. `/api/v2/pokemon/25/`.

            // Strip the `/` suffix identified earlier
            let path_str = path.strip_suffix("/").unwrap();

            // Split path into the folder name and everything before it
            let (endpoint, identifier) = path_str.rsplit_once('/').context(BadPathSnafu {
                message: "no slash found",
                path: path_str,
            })?;

            // Append index.json and return if folder name is a number.
            if identifier.parse::<u64>().is_ok() {
                return Ok(RelativePath::new(path).join("index.json"));
            }

            // Representation of what's found in index.json
            #[derive(serde::Deserialize)]
            struct Index {
                results: Vec<Entry>,
            }
            #[derive(serde::Deserialize)]
            struct Entry {
                name: String,
                #[serde(rename = "url")]
                path: String,
            }

            // Append index.json to the endpoint path to get Index.
            let path_to_index = RelativePath::new(endpoint).join("index.json");
            let index =
                Self::serialize::<Index>(self.transport.read(&path_to_index).await?).await?;

            // Find the entry within the index that corresponds to our identifier.
            let entry = index
                .results
                .into_iter()
                .find(|entry| entry.name == identifier)
                .context(NotFoundInIndexSnafu {
                    identifier,
                    path_to_index,
                })?;

            // Turn extracted path that looks like `/api/v2/pokemon/25/` into `api/v2/pokemon/25/index.json
            let target_path_str = entry.path.strip_prefix("/").unwrap_or(path);
            RelativePath::new(target_path_str).join("index.json")
        } else {
            // Path points to a file. Return valid path directly.
            RelativePath::new(path).to_owned()
        };

        Ok(path)
    }
}
