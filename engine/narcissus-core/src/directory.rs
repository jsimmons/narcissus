use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Returns the path to the user's cache directory.
///
/// | Platform |                                    |
/// | -------- | -----------------------------------|
/// | Linux    | `$XDG_CACHE_DIR` or `$HOME/.cache` |
pub fn cache_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<&'static Path>> = OnceLock::new();

    #[cfg(target_os = "linux")]
    *DIR.get_or_init(|| {
        let path = if let Some(path) = std::env::var_os("XDG_CACHE_DIR") {
            path.into()
        } else {
            let mut path: PathBuf = std::env::var_os("HOME")?.into();
            path.push(".cache");
            path
        };
        Some(Box::leak(path.into_boxed_path()))
    })
}

/// Returns the path to the user's config directory.
///
/// | Platform |                                       |
/// | -------- | --------------------------------------|
/// | Linux    | `$XDG_CONFIG_HOME` or `$HOME/.config` |
pub fn config_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<&'static Path>> = OnceLock::new();

    #[cfg(target_os = "linux")]
    *DIR.get_or_init(|| {
        let path = if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            path.into()
        } else {
            let mut path: PathBuf = std::env::var_os("HOME")?.into();
            path.push(".config");
            path
        };
        Some(Box::leak(path.into_boxed_path()))
    })
}

/// Returns the path to the user's data directory.
///
/// | Platform |                                          |
/// | -------- | -----------------------------------------|
/// | Linux    | `$XDG_DATA_HOME` or `$HOME/.local/share` |
pub fn data_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<&'static Path>> = OnceLock::new();

    #[cfg(target_os = "linux")]
    *DIR.get_or_init(|| {
        let path = if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
            path.into()
        } else {
            let mut path: PathBuf = std::env::var_os("HOME")?.into();
            path.push(".local");
            path.push("share");
            path
        };
        Some(Box::leak(path.into_boxed_path()))
    })
}

/// Returns the path to the user's runtime directory.
///
/// | Platform |                    |
/// | -------- | -------------------|
/// | Linux    | `$XDG_RUNTIME_DIR` |
pub fn runtime_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<&'static Path>> = OnceLock::new();

    #[cfg(target_os = "linux")]
    *DIR.get_or_init(|| {
        let path: PathBuf = std::env::var_os("XDG_RUNTIME_DIR")?.into();
        Some(Box::leak(path.into_boxed_path()))
    })
}
