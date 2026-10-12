//! Bounded acquisition receipts and errors contain no file content.

use serde::Serialize;
use std::fmt;

/// Content-free receipt for exact validated report bytes.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MountedSharePointReportReceipt {
    /// Versioned contract URI.
    pub schema: &'static str,
    /// Opaque identity of this request.
    pub request_id: String,
    /// Receipt id.
    pub receipt_id: String,
    /// Correlation id.
    pub correlation_id: String,
    /// Opaque identifier of an explicitly selected root.
    pub root_id: String,
    /// Artifact id.
    pub artifact_id: String,
    /// Operational report id.
    pub operational_report_id: String,
    /// Headquarters id.
    pub headquarters_id: String,
    /// Department id.
    pub department_id: String,
    /// Reporting team id.
    pub reporting_team_id: String,
    /// Versioned schema identifier of the referenced projection.
    pub schema_id: &'static str,
    /// Media type.
    pub media_type: &'static str,
    /// Size bytes.
    pub size_bytes: u64,
    /// Lowercase SHA-256 digest of the referenced projection bytes.
    pub digest_sha256: String,
    /// Classification.
    pub classification: String,
    /// Customer data.
    pub customer_data: bool,
    /// External actions.
    pub external_actions: bool,
}

/// Stable fail-closed acquisition error.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReportAcquisitionError {
    /// Code.
    pub code: &'static str,
    /// Field.
    pub field: &'static str,
    /// Message.
    pub message: &'static str,
}

impl fmt::Display for ReportAcquisitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ReportAcquisitionError {}
