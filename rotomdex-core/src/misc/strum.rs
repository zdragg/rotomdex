/// Wrapper for [`strum::ParseError`]. [`strum`] maintains compatibilty
/// beyond Rust 1.81.0, meaning it only implements [`std::error::Error`].
#[derive(Debug)]
pub(crate) struct StrumParseError(pub strum::ParseError);

impl core::fmt::Display for StrumParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl core::error::Error for StrumParseError {}
