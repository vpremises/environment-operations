//! The allowlisted root must stay beneath its explicit base with no symlinked prefix.

mod path;
use path::{reject_symlinked_prefix, validate_base, validate_relative};

use crate::{
    model::{ReportAcquisitionError, ReportAllowedRoot},
    report::error,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn validate(
    root: &ReportAllowedRoot,
    resolved_base: &Path,
) -> Result<PathBuf, ReportAcquisitionError> {
    validate_base(resolved_base)?;
    let relative_root = validate_relative(&root.relative_path)?;
    reject_symlinked_prefix(resolved_base)?;
    let canonical_base = resolved_base.canonicalize().map_err(|_| {
        error::create(
            "vpremises.report.root-base-unavailable",
            "$.allowlisted_root.base",
            "the resolved root base is unavailable",
        )
    })?;
    let root_path = canonical_base.join(relative_root);
    reject_symlinked_prefix(&root_path)?;
    let metadata = fs::symlink_metadata(&root_path).map_err(|_| {
        error::create(
            "vpremises.report.root-unavailable",
            "$.allowlisted_root.relative_path",
            "the allowlisted root is unavailable",
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(error::create(
            "vpremises.report.root-invalid",
            "$.allowlisted_root.relative_path",
            "the allowlisted root must be a regular directory and not a symbolic link",
        ));
    }
    let canonical = root_path.canonicalize().map_err(|_| {
        error::create(
            "vpremises.report.root-unavailable",
            "$.allowlisted_root.relative_path",
            "the allowlisted root could not be resolved",
        )
    })?;
    if canonical == Path::new("/") || !canonical.starts_with(&canonical_base) {
        return Err(error::create(
            "vpremises.report.root-invalid",
            "$.allowlisted_root.relative_path",
            "the allowlisted root must remain below its resolved base",
        ));
    }
    Ok(canonical)
}
