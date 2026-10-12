//! Private temporary output and directory-anchored no-follow input access.
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
/// Runtime scratch is private and removed on every return path.
pub(crate) struct Scratch(pub PathBuf);
impl Scratch {
    pub(crate) fn create() -> Result<Self, &'static str> {
        use std::os::unix::fs::DirBuilderExt;
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "vpremises-audit-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::DirBuilder::new().mode(0o700).create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err("scratch-unavailable"),
            }
        }
        Err("scratch-unavailable")
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Read one physical regular file with fixed byte and change-detection bounds.
pub(crate) fn read(path: &Path, maximum: u64) -> Result<Vec<u8>, &'static str> {
    use std::os::unix::fs::MetadataExt;
    let file = open(path)?;
    let before = file.metadata().map_err(|_| "input-unavailable")?;
    if !before.is_file() || before.len() > maximum {
        return Err("input-boundary");
    }
    let mut bytes = Vec::new();
    (&file)
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "input-read-failed")?;
    let after = file.metadata().map_err(|_| "input-unavailable")?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > maximum
        || (
            before.len(),
            before.mtime(),
            before.mtime_nsec(),
            before.ctime(),
            before.ctime_nsec(),
        ) != (
            after.len(),
            after.mtime(),
            after.mtime_nsec(),
            after.ctime(),
            after.ctime_nsec(),
        )
    {
        return Err("input-changed");
    }
    Ok(bytes)
}
/// Anchor every path component below the Linux root without following links.
pub(crate) fn open(path: &Path) -> Result<File, &'static str> {
    if !path.is_absolute() {
        return Err("absolute-path-required");
    }
    crate::report::open::beneath(
        Path::new("/"),
        path.strip_prefix("/").map_err(|_| "input-boundary")?,
    )
    .map_err(|_| "input-boundary")
}
/// Never reuse or truncate an existing runtime output.
pub(super) fn output(path: &Path) -> Result<File, &'static str> {
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "output-unavailable")
}
