//! Validate portable acquisition paths and reject linked ancestors.

use crate::{
    model::ReportAcquisitionError, report::error, MAX_RELATIVE_COMPONENTS, MAX_RELATIVE_FILE_BYTES,
};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(super) fn validate_base(path: &Path) -> Result<(), ReportAcquisitionError> {
    if !path.is_absolute()
        || path == Path::new("/")
        || path
            .components()
            .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
    {
        return Err(error::create(
            "vpremises.report.root-base-invalid",
            "$.allowlisted_root.base",
            "the resolved root base must be an absolute non-system path",
        ));
    }
    Ok(())
}

pub(super) fn validate_relative(path: &Path) -> Result<&Path, ReportAcquisitionError> {
    let count = path.components().count();
    let portable = path.to_str().is_some_and(|value| {
        value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
    });
    if path.as_os_str().as_encoded_bytes().is_empty()
        || path.as_os_str().as_encoded_bytes().len() > MAX_RELATIVE_FILE_BYTES
        || !portable
        || path.is_absolute()
        || count == 0
        || count > MAX_RELATIVE_COMPONENTS
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(error::create(
            "vpremises.report.root-relative-path-invalid",
            "$.allowlisted_root.relative_path",
            "relative_path must be a bounded relative path without traversal",
        ));
    }
    Ok(path)
}

pub(super) fn reject_symlinked_prefix(path: &Path) -> Result<(), ReportAcquisitionError> {
    let mut current = PathBuf::from("/");
    for component in path.components() {
        if let Component::Normal(name) = component {
            current.push(name);
            let metadata = fs::symlink_metadata(&current).map_err(|_| {
                error::create(
                    "vpremises.report.root-unavailable",
                    "$.allowlisted_root.relative_path",
                    "an allowlisted root path component is unavailable",
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(error::create(
                    "vpremises.report.root-invalid",
                    "$.allowlisted_root.relative_path",
                    "symbolic links are forbidden in the allowlisted root path",
                ));
            }
        }
    }
    Ok(())
}
