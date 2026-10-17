//! Bounded filesystem access for configuration documents.

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use nalios_app::config::{ConfigSource, ReadError};
use nalios_core::config::MAX_CONFIG_BYTES;

/// Reads configuration documents from the filesystem with a hard size limit.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsConfigSource;

impl ConfigSource for FsConfigSource {
    fn read(&self, path: &Path) -> Result<String, ReadError> {
        let file = File::open(path).map_err(map_open_error)?;
        read_bounded(file)
    }
}

/// The default user configuration path for the development namespace.
///
/// Returns `None` when neither `XDG_CONFIG_HOME` nor `HOME` yields an absolute
/// base directory.
pub fn default_config_path() -> Option<PathBuf> {
    config_path_from(
        std::env::var_os("XDG_CONFIG_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
    )
}

fn config_path_from(config_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    let base = match config_home {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => {
            let home = home?;
            if home.is_empty() {
                return None;
            }
            PathBuf::from(home).join(".config")
        }
    };

    if !base.is_absolute() {
        return None;
    }

    Some(base.join("nalios-dev").join("config.toml"))
}

fn read_bounded(file: File) -> Result<String, ReadError> {
    let mut bytes = Vec::new();
    let mut reader = file.take(MAX_CONFIG_BYTES as u64 + 1);
    reader
        .read_to_end(&mut bytes)
        .map_err(|_| ReadError::Unreadable)?;

    if bytes.len() > MAX_CONFIG_BYTES {
        return Err(ReadError::TooLarge);
    }

    String::from_utf8(bytes).map_err(|_| ReadError::NotUtf8)
}

fn map_open_error(error: std::io::Error) -> ReadError {
    match error.kind() {
        std::io::ErrorKind::NotFound => ReadError::NotFound,
        std::io::ErrorKind::PermissionDenied => ReadError::PermissionDenied,
        _ => ReadError::Unreadable,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    fn write_temp(bytes: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("temp file");
        file.write_all(bytes).expect("write");
        file.flush().expect("flush");
        file
    }

    #[test]
    fn reads_a_valid_document() {
        let file = write_temp(b"schema_version = 1\n");

        let document = FsConfigSource.read(file.path()).expect("reads");

        assert_eq!(document, "schema_version = 1\n");
    }

    #[test]
    fn missing_file_is_not_found() {
        let path = std::path::PathBuf::from("/definitely/missing/nalios-config.toml");

        assert!(matches!(
            FsConfigSource.read(&path),
            Err(ReadError::NotFound)
        ));
    }

    #[test]
    fn oversized_file_is_rejected() {
        let bytes = vec![b'x'; MAX_CONFIG_BYTES + 1];
        let file = write_temp(&bytes);

        assert!(matches!(
            FsConfigSource.read(file.path()),
            Err(ReadError::TooLarge)
        ));
    }

    #[test]
    fn invalid_utf8_is_rejected() {
        let file = write_temp(&[0xff, 0xfe, 0xfd]);

        assert!(matches!(
            FsConfigSource.read(file.path()),
            Err(ReadError::NotUtf8)
        ));
    }

    #[test]
    fn config_path_prefers_xdg_config_home() {
        assert_eq!(
            config_path_from(
                Some(OsStr::new("/custom/config")),
                Some(OsStr::new("/home/tester"))
            ),
            Some(PathBuf::from("/custom/config/nalios-dev/config.toml"))
        );
        assert_eq!(
            config_path_from(None, Some(OsStr::new("/home/tester"))),
            Some(PathBuf::from("/home/tester/.config/nalios-dev/config.toml"))
        );
        assert_eq!(
            config_path_from(Some(OsStr::new("relative")), Some(OsStr::new("/home/tester"))),
            None
        );
        assert_eq!(config_path_from(None, None), None);
    }

    #[test]
    fn open_error_kinds_are_mapped() {
        let not_found = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let denied = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let other = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "other");

        assert!(matches!(map_open_error(not_found), ReadError::NotFound));
        assert!(matches!(
            map_open_error(denied),
            ReadError::PermissionDenied
        ));
        assert!(matches!(map_open_error(other), ReadError::Unreadable));
    }
}
