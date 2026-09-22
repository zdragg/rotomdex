use alloc::{boxed::Box, string::String};
use relative_path::RelativePathBuf;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    JsonSerialize { source: serde_json::Error },

    #[snafu(visibility(pub(crate)), display("{path} is invalid: {message}"))]
    BadPath {
        message: String,
        path: RelativePathBuf,
    },

    /// Example: "pikucha not found under api/v2/pokemon-species/index.json"
    #[snafu(
        visibility(pub(crate)),
        display("{identifier} not found in {path_to_index}")
    )]
    NotFoundInIndex {
        identifier: String,
        path_to_index: RelativePathBuf,
    },

    #[snafu(transparent)]
    Transport { source: TransportError },
}

/// Construct `NotFoundSnafu` if your transport cannot find a file that should exist
/// (e.g. the file requested or an intermediate `index.json`).
/// If your transport could produce other errors (e.g. I/O errors),
/// define your own Error enum and implement
/// [`OtherTransportError`] for it.
/// Then, all variants of your Error enum would be able to
/// convert into `TransportError` automagically.
#[derive(Debug, Snafu)]
pub enum TransportError {
    #[snafu(visibility(pub), display("{path} not found"))]
    NotFound { path: RelativePathBuf },

    #[snafu(transparent)]
    Other {
        source: Box<dyn core::error::Error + Send + Sync + 'static>,
    },
}

/// Implement this trait for your own Error enum so it can be converted into [`TransportError`].
pub trait OtherTransportError: core::error::Error + Send + Sync + 'static {}

impl<E: OtherTransportError> From<E> for TransportError {
    fn from(value: E) -> Self {
        Self::Other {
            source: Box::new(value),
        }
    }
}
