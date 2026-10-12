//! Linux opens every component relative to a held descriptor with no-follow semantics.

use crate::{model::ReportAcquisitionError, report::error};
use std::{fs, path::Path};

#[cfg(target_os = "linux")]
use std::{
    fs::OpenOptions,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Component, PathBuf},
};

#[cfg(target_os = "linux")]
pub(crate) fn beneath(
    canonical_root: &Path,
    relative: &Path,
) -> Result<fs::File, ReportAcquisitionError> {
    const O_DIRECTORY: i32 = 0o200_000;
    const O_NOFOLLOW: i32 = 0o400_000;
    const O_NONBLOCK: i32 = 0o4_000;
    let mut directory = OpenOptions::new()
        .read(true)
        .custom_flags(O_DIRECTORY | O_NOFOLLOW)
        .open("/")
        .map_err(|_| root_open_error())?;
    for component in canonical_root.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                directory = OpenOptions::new()
                    .read(true)
                    .custom_flags(O_DIRECTORY | O_NOFOLLOW)
                    .open(descriptor_child(&directory, name))
                    .map_err(|_| root_open_error())?;
            }
            _ => return Err(root_open_error()),
        }
    }
    let components = relative.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err(relative_error());
        };
        let child = descriptor_child(&directory, name);
        if index + 1 == components.len() {
            return OpenOptions::new()
                .read(true)
                .custom_flags(O_NOFOLLOW | O_NONBLOCK)
                .open(child)
                .map_err(|_| {
                    error::create(
                        "vpremises.report.open-failed",
                        "$.relative_file",
                        "the allowlisted report could not be opened with no-follow semantics",
                    )
                });
        }
        directory = OpenOptions::new()
            .read(true)
            .custom_flags(O_DIRECTORY | O_NOFOLLOW)
            .open(child)
            .map_err(|_| {
                error::create(
                    "vpremises.report.path-open-failed",
                    "$.relative_file",
                    "a report directory could not be opened with no-follow semantics",
                )
            })?;
    }
    Err(relative_error())
}

#[cfg(target_os = "linux")]
fn descriptor_child(directory: &fs::File, name: &std::ffi::OsStr) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd())).join(name)
}

#[cfg(target_os = "linux")]
fn root_open_error() -> ReportAcquisitionError {
    error::create(
        "vpremises.report.root-open-failed",
        "$.allowlisted_root.relative_path",
        "the allowlisted root could not be opened with no-follow semantics",
    )
}

#[cfg(target_os = "linux")]
fn relative_error() -> ReportAcquisitionError {
    error::create(
        "vpremises.report.relative-file-invalid",
        "$.relative_file",
        "relative_file must remain below the allowlisted root",
    )
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn beneath(
    _canonical_root: &Path,
    _relative: &Path,
) -> Result<fs::File, ReportAcquisitionError> {
    Err(error::create(
        "vpremises.report.secure-open-unsupported",
        "$.relative_file",
        "mounted report acquisition requires a directory-anchored no-follow open",
    ))
}
