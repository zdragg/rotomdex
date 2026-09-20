use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("resource not found")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
