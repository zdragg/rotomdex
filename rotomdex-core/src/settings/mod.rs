mod version;
pub use version::*;

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub version: version::Version,
    pub animation_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: Version::default(),
            animation_mode: true,
        }
    }
}
