use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub(super) fn create_session_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

pub(super) fn store_candidate(path: &Path, bytes: &[u8]) -> io::Result<()> {
    create_immutable_file(path, bytes)
}

pub(super) fn persist_manifest(path: &Path, bytes: &[u8]) -> io::Result<()> {
    create_immutable_file(&path.join("manifest.json"), bytes)
}

pub(super) fn clean_failed_session(session_dir: &Path, candidate_id: Option<&str>) {
    if let Some(candidate_id) = candidate_id {
        let _ = fs::remove_file(session_dir.join(format!("{candidate_id}.png")));
    }
    let _ = fs::remove_file(session_dir.join("manifest.json"));
    let _ = fs::remove_dir(session_dir);
}

#[cfg(target_os = "macos")]
pub(super) fn default_candidate_dir() -> io::Result<PathBuf> {
    Ok(home_dir()?.join("Library/Caches/image-optimizer"))
}

#[cfg(target_os = "linux")]
pub(super) fn default_candidate_dir() -> io::Result<PathBuf> {
    let xdg_cache_home = std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());
    let cache_home = match xdg_cache_home {
        Some(cache_home) => cache_home,
        None => home_dir()?.join(".cache"),
    };
    Ok(cache_home.join("image-optimizer"))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(super) fn default_candidate_dir() -> io::Result<PathBuf> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "default candidate storage is supported only on macOS and Linux",
    ))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn home_dir() -> io::Result<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; use --candidate-dir to select candidate storage",
        )
    })
}

fn create_immutable_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_readonly(true);
        file.set_permissions(permissions)
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}
