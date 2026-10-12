//! Mounted-report contracts bind one expected artifact to one allowlisted local root.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Closed request binding one report to an expected organizational context.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MountedSharePointReportRequest {
    /// Versioned contract URI.
    pub schema: String,
    /// Opaque identity of this request.
    pub request_id: String,
    /// Correlation id.
    pub correlation_id: String,
    /// Artifact id.
    pub artifact_id: String,
    /// Expected report.
    pub expected_report: ExpectedDepartmentDailyReport,
    /// Allowlisted root.
    pub allowlisted_root: ReportAllowedRoot,
    /// Relative file.
    pub relative_file: String,
    /// Max bytes.
    pub max_bytes: u64,
}

/// Portable root reference resolved outside the acquisition library.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReportAllowedRoot {
    /// Id.
    pub id: String,
    /// Base.
    pub base: ReportRootBase,
    /// Relative path.
    pub relative_path: PathBuf,
}

/// Base from which the portable root path is resolved by the caller.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReportRootBase {
    /// Resolve from a registered workspace root.
    WorkspaceRoot,
    /// Resolve from the request document directory.
    RequestDirectory,
}

/// Identity and information policy expected in the report body.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExpectedDepartmentDailyReport {
    /// Record mode.
    pub record_mode: ReportRecordMode,
    /// Reporting date.
    pub reporting_date: String,
    /// Headquarters id.
    pub headquarters_id: String,
    /// Department id.
    pub department_id: String,
    /// Reporting team id.
    pub reporting_team_id: String,
    /// Owner account id.
    pub owner_account_id: String,
    /// Classification.
    pub classification: ReportClassification,
    /// Customer data.
    pub customer_data: bool,
}

/// Supported operational record modes.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReportRecordMode {
    /// Synthetic data used for verification.
    SimulationSeed,
    /// Data produced by a live runtime.
    Runtime,
    /// Data acquired from an external source.
    Imported,
}

/// Supported information classifications.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReportClassification {
    /// Information intended for internal use.
    Internal,
    /// Confidential internal information.
    InternalConfidential,
    /// Information containing protected personal data.
    PersonalConfidential,
    /// Information requiring restricted handling.
    RestrictedSensitive,
    /// Confidential customer information.
    CustomerConfidential,
}
