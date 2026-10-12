//! Mounted-report acquisition returns bounded metadata after identity and content validation.

mod document;
mod error;
pub(crate) mod open;
mod path;
mod receipt;
mod request;
mod root;

use crate::model::{
    MountedSharePointReportReceipt, MountedSharePointReportRequest, ReportAcquisitionError,
};
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path};

/// Acquires one allowlisted mounted report through a no-follow boundary.
///
/// # Errors
///
/// Returns a structured error when the request, root, relative path, opened
/// file, byte limit, closed report document, or expected identity is invalid.
pub fn acquire(
    request: &MountedSharePointReportRequest,
    resolved_root_base: &Path,
) -> Result<MountedSharePointReportReceipt, ReportAcquisitionError> {
    request::validate(request)?;
    let canonical_root = root::validate(&request.allowlisted_root, resolved_root_base)?;
    let relative = path::relative_report_file(&request.relative_file)?;
    let report_path = canonical_root.join(&relative);
    path::validate_report_path(&canonical_root, &report_path)?;
    let file = open::beneath(&canonical_root, &relative)?;
    let metadata = file.metadata().map_err(|_| {
        error::create(
            "vpremises.report.file-unavailable",
            "$.relative_file",
            "the allowlisted report file is unavailable",
        )
    })?;
    require_regular_and_bounded(&metadata, request.max_bytes)?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len().min(request.max_bytes)).unwrap_or_default(),
    );
    file.take(request.max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| {
            error::create(
                "vpremises.report.read-failed",
                "$.relative_file",
                "the allowlisted report could not be read",
            )
        })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > request.max_bytes {
        return Err(size_error());
    }
    path::validate_report_path(&canonical_root, &report_path)?;
    let report = document::validate(&bytes)?;
    if request.artifact_id != report.operational_report_id {
        return Err(error::create(
            "vpremises.report.artifact-id-mismatch",
            "$.artifact_id",
            "artifact_id must equal the report operational_report_id",
        ));
    }
    document::validate_expected(&request.expected_report, &report)?;
    let digest_sha256 = receipt::lower_hex(&Sha256::digest(&bytes));
    Ok(receipt::build(
        request,
        report,
        digest_sha256,
        u64::try_from(bytes.len()).unwrap_or(u64::MAX),
    ))
}

fn require_regular_and_bounded(
    metadata: &std::fs::Metadata,
    maximum: u64,
) -> Result<(), ReportAcquisitionError> {
    if !metadata.is_file() {
        return Err(error::create(
            "vpremises.report.regular-file-required",
            "$.relative_file",
            "the allowlisted report must be a regular file",
        ));
    }
    if metadata.len() > maximum {
        return Err(size_error());
    }
    Ok(())
}

fn size_error() -> ReportAcquisitionError {
    error::create(
        "vpremises.report.size-exceeded",
        "$.max_bytes",
        "the report exceeds the request byte limit",
    )
}
