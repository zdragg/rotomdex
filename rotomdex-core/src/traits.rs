pub use rotomdex_api::client::{CachedTransport, Error as ApiError, RelativePath, RelativePathBuf};

use crate::Session;

pub trait SessionRw {
    fn read(&self) -> Session;
    fn write(&self, session: Session);
}
