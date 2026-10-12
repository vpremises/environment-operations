//! Hold each enumerated directory; never re-follow a mutable pathname.
use std::{
    fs::{File, OpenOptions},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Path, PathBuf},
};

pub(super) fn root(path: &Path) -> Result<File, &'static str> {
    let directory = crate::report::open::beneath(
        Path::new("/"),
        path.strip_prefix("/").map_err(|_| "unsafe root")?,
    )
    .map_err(|_| "unsafe root")?;
    if !directory.metadata().map_err(|_| "unsafe root")?.is_dir() {
        return Err("unsafe root");
    }
    Ok(directory)
}
pub(super) fn path(directory: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()))
}
pub(super) fn child(path: &Path) -> std::io::Result<File> {
    // The parent is a held /proc descriptor, not a caller-provided path.
    const O_DIRECTORY: i32 = 0o200_000;
    const O_NOFOLLOW: i32 = 0o400_000;
    OpenOptions::new()
        .read(true)
        .custom_flags(O_DIRECTORY | O_NOFOLLOW)
        .open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renamed_directory_cannot_redirect_a_held_inventory_to_an_outside_link() {
        let base = crate::audit::io::Scratch::create().unwrap();
        let selected = base.0.join("selected");
        let outside = base.0.join("outside");
        std::fs::create_dir(&selected).unwrap();
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(selected.join("inside"), b"safe").unwrap();
        std::fs::write(outside.join("outside"), b"safe").unwrap();
        let held = root(&selected).unwrap();
        std::fs::rename(&selected, base.0.join("moved")).unwrap();
        std::os::unix::fs::symlink(&outside, &selected).unwrap();
        let entries = std::fs::read_dir(path(&held))
            .unwrap()
            .map(|x| x.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(entries, vec![std::ffi::OsString::from("inside")]);
        assert!(child(&selected).is_err());
    }
}
