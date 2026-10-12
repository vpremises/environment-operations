//! Report paths are bounded JSON descendants and are revalidated around the actual read.

use crate::{
    model::ReportAcquisitionError, report::error, MAX_RELATIVE_COMPONENTS, MAX_RELATIVE_FILE_BYTES,
};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(super) fn relative_report_file(value: &str) -> Result<PathBuf, ReportAcquisitionError> {
    let path = Path::new(value);
    let count = path.components().count();
    if value.is_empty()
        || value.len() > MAX_RELATIVE_FILE_BYTES
        || !portable(value)
        || path.is_absolute()
        || count == 0
        || count > MAX_RELATIVE_COMPONENTS
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || path.extension().and_then(|extension| extension.to_str()) != Some("json")
    {
        return Err(error::create(
            "vpremises.report.relative-file-invalid",
            "$.relative_file",
            "relative_file must be a bounded relative .json path without traversal",
        ));
    }
    Ok(path.to_path_buf())
}

fn portable(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
}

pub(super) fn validate_report_path(
    canonical_root: &Path,
    report_path: &Path,
) -> Result<(), ReportAcquisitionError> {
    let relative = report_path
        .strip_prefix(canonical_root)
        .map_err(|_| boundary_error())?;
    let component_count = relative.components().count();
    let mut current = canonical_root.to_path_buf();
    for (index, component) in relative.components().enumerate() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current).map_err(|_| {
            error::create(
                "vpremises.report.path-unavailable",
                "$.relative_file",
                "a report path component is unavailable",
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(error::create(
                "vpremises.report.symlink-forbidden",
                "$.relative_file",
                "symbolic links are forbidden in the report path",
            ));
        }
        let final_component = index + 1 == component_count;
        if (!final_component && !metadata.is_dir()) || (final_component && !metadata.is_file()) {
            return Err(error::create(
                "vpremises.report.path-kind-invalid",
                "$.relative_file",
                "the report path contains an unexpected entry kind",
            ));
        }
    }
    let canonical_report = report_path.canonicalize().map_err(|_| {
        error::create(
            "vpremises.report.path-unavailable",
            "$.relative_file",
            "the report path could not be resolved",
        )
    })?;
    if !canonical_report.starts_with(canonical_root) {
        return Err(boundary_error());
    }
    Ok(())
}

fn boundary_error() -> ReportAcquisitionError {
    error::create(
        "vpremises.report.boundary-escape",
        "$.relative_file",
        "the report path leaves its allowlisted root",
    )
}
