use std::{fs, io, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::Version;

#[derive(Serialize, Deserialize)]
struct Config {
    version: String,
}

/// Reads the persisted version from the JSON config file.
///
/// Returns `None` when the file is missing or does not hold a valid version so
/// that the caller can fall back to the default version.
pub(crate) fn load_version(path: &Path) -> Option<Version> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => {
            warn!(?error, path = %path.display(), "failed to read config");
            return None;
        }
    };

    let config = match serde_json::from_str::<Config>(&contents) {
        Ok(config) => config,
        Err(error) => {
            warn!(?error, path = %path.display(), "failed to parse config");
            return None;
        }
    };

    match Version::from_str(&config.version) {
        Ok(version) => Some(version),
        Err(_) => {
            warn!(path = %path.display(), version = %config.version, "config contains an unknown version");
            None
        }
    }
}

/// Writes the selected version to the config file, creating its directory if needed.
pub(crate) fn save_version(path: &Path, version: Version) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut contents = serde_json::to_string_pretty(&Config {
        version: version.to_string(),
    })
    .map_err(io::Error::other)?;
    contents.push('\n');

    // Write to a temporary file and rename it into place so an interrupted write
    // cannot leave a truncated config behind.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rotomdex-config-test-{}-{name}", std::process::id()));
        dir.join("config.json")
    }

    #[test]
    fn saves_and_loads_version_roundtrip() {
        let path = temp_path("roundtrip");

        save_version(&path, Version::LegendsZa).unwrap();
        assert_eq!(load_version(&path), Some(Version::LegendsZa));

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn ignores_missing_or_invalid_configs() {
        let path = temp_path("invalid");
        let dir = path.parent().unwrap();

        // Missing file.
        assert_eq!(load_version(&path), None);

        fs::create_dir_all(dir).unwrap();
        for contents in [r#"not json"#, r#"{"version":"not-a-version"}"#, r#"{"other":"sword"}"#] {
            fs::write(&path, contents).unwrap();
            assert_eq!(load_version(&path), None, "contents: {contents}");
        }

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn accepts_extra_keys() {
        let path = temp_path("extra");
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir).unwrap();
        fs::write(&path, "{\n  \"version\": \"soulsilver\",\n  \"future_key\": true\n}").unwrap();

        assert_eq!(load_version(&path), Some(Version::SoulSilver));

        std::fs::remove_dir_all(dir).unwrap();
    }
}
