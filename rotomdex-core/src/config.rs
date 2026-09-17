use std::{fs, io, path::Path, str::FromStr};

use tracing::warn;

use crate::Version;

/// Reads the persisted version from the config file.
///
/// Returns `None` when the file is missing or does not hold a valid version so
/// that the caller can fall back to the default version.
pub(crate) fn load_version(path: &Path) -> Option<Version> {
    match fs::read_to_string(path) {
        Ok(contents) => match parse_version(&contents) {
            Some(version) => Some(version),
            None => {
                warn!(path = %path.display(), "config contains no valid version");
                None
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            warn!(?error, path = %path.display(), "failed to read config");
            None
        }
    }
}

/// Writes the selected version to the config file, creating its directory if needed.
pub(crate) fn save_version(path: &Path, version: Version) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, format!("# rotomdex configuration\nversion = \"{version}\"\n"))
}

fn parse_version(contents: &str) -> Option<Version> {
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "version" {
            return Version::from_str(value.trim().trim_matches('"')).ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_and_bare_versions() {
        assert_eq!(parse_version("# comment\nversion = \"scarlet\"\n"), Some(Version::Scarlet));
        assert_eq!(parse_version("version=black-2"), Some(Version::Black2));
    }

    #[test]
    fn ignores_missing_or_invalid_versions() {
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("other = \"sword\""), None);
        assert_eq!(parse_version("version = \"not-a-version\""), None);
    }

    #[test]
    fn saves_and_loads_version_roundtrip() {
        let dir = std::env::temp_dir().join(format!("rotomdex-config-test-{}", std::process::id()));
        let path = dir.join("config.toml");

        save_version(&path, Version::LegendsZa).unwrap();
        assert_eq!(load_version(&path), Some(Version::LegendsZa));

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
