//! Acquisition request loading bounds the file before resolving its declared root base.

use super::output;
use serde_json::Value;
use std::path::{Path, PathBuf};
use vpremises::{MountedSharePointReportRequest, ReportRootBase};

const MAX_REQUEST_BYTES: u64 = 64 * 1024;

pub(super) fn load(path: &Path) -> Result<MountedSharePointReportRequest, (u8, Value)> {
    let input = super::file_input::read_utf8(path, MAX_REQUEST_BYTES).map_err(|message| {
        (
            1,
            output::report_request_error("vpremises.report-request.unsafe-file", message),
        )
    })?;
    serde_json::from_str(&input).map_err(|error| {
        (
            1,
            output::report_request_error(
                "vpremises.report-request.decode",
                &format!(
                    "invalid request JSON at line {}, column {}",
                    error.line(),
                    error.column()
                ),
            ),
        )
    })
}

pub(super) fn resolve_root_base(
    request_path: &Path,
    base: ReportRootBase,
) -> Result<PathBuf, (u8, Value)> {
    let canonical_request = request_path.canonicalize().map_err(|error| {
        (
            1,
            output::report_request_error(
                "vpremises.report-request.base-unavailable",
                &format!("request base could not be resolved: {}", error.kind()),
            ),
        )
    })?;
    let request_directory = canonical_request.parent().ok_or_else(|| {
        (
            1,
            output::report_request_error(
                "vpremises.report-request.base-unavailable",
                "request file has no resolvable parent directory",
            ),
        )
    })?;
    match base {
        ReportRootBase::RequestDirectory => Ok(request_directory.to_owned()),
        ReportRootBase::WorkspaceRoot => {
            super::workspace::find_root(request_directory).ok_or_else(|| {
                (
                    1,
                    output::report_request_error(
                        "vpremises.report-request.workspace-root-unavailable",
                        "workspace-root requires an ancestor with a regular source-foundation.toml",
                    ),
                )
            })
        }
    }
}
