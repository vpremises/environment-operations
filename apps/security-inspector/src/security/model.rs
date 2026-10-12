//! Portable reports separate observed metadata from security-check coverage.

use crate::ObservationReport;
use serde::Serialize;

/// A local, consumer-independent endpoint report.
#[derive(Debug, Serialize)]
pub struct SecurityReport {
    /// Versioned report contract.
    pub schema: &'static str,
    /// Executable package version.
    pub tool_version: &'static str,
    /// Operator-assigned environment identifier; never a hostname.
    pub environment_id: String,
    /// Native operating system of this process.
    pub operating_system: &'static str,
    /// UTC collection start, milliseconds since the Unix epoch.
    pub started_at_unix_ms: u64,
    /// UTC collection end, milliseconds since the Unix epoch.
    pub finished_at_unix_ms: u64,
    /// SHA-256 of the effective observer configuration.
    pub configuration_sha256: String,
    /// Overall coverage outcome, independent of successful metadata collection.
    pub outcome: &'static str,
    /// Whether this run made any external changes.
    pub external_actions: bool,
    /// Coverage of each declared check.
    pub checks: Vec<CheckCoverage>,
    /// Aggregate-only metadata observation.
    pub observation: ObservationReport,
}

/// One check's collection status and bounded explanation.
#[derive(Debug, Serialize)]
pub struct CheckCoverage {
    /// Stable identifier for the check.
    pub id: &'static str,
    /// Collection status, independent of security findings.
    pub status: &'static str,
    /// Stable reason code, with no raw input or private values.
    pub reason: &'static str,
    /// Explicit evidence scope; mounted Windows files do not imply host observation.
    pub scope: &'static str,
    /// Number of findings without raw paths or detected values.
    pub finding_count: u64,
    /// Number of known coverage gaps.
    pub gap_count: u64,
    /// Digest of local engine evidence; raw evidence is never embedded.
    pub evidence_sha256: Option<String>,
}
