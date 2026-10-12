//! Reports contain stable reasons and relative locations, never matched contents.
use serde::Serialize;

/// One policy finding or an uninspected location.
#[derive(Debug, Serialize)]
pub struct OssFinding {
    /// Stable reason identifier.
    pub code: &'static str,
    /// Relative filename or manifest field; empty for root-level errors.
    pub location: String,
}

/// A bounded, local OSS inspection receipt, separate from security certification.
#[derive(Debug, Serialize)]
pub struct OssReport {
    /// Transport-neutral report contract.
    pub schema: &'static str,
    /// Selected gate, not inferred whole-device scope.
    pub gate: &'static str,
    /// Passed, findings, or incomplete.
    pub status: &'static str,
    /// Actual policy failures.
    pub findings: Vec<OssFinding>,
    /// Failures to inspect are never a pass.
    pub uninspected: Vec<OssFinding>,
    /// Hashes bind input files without exposing their contents.
    pub evidence_sha256: Vec<String>,
    /// Explicitly state that no network, install, or publish action occurs.
    pub external_actions: bool,
}

impl OssReport {
    pub(super) fn new(gate: &'static str) -> Self {
        Self {
            schema: "vpremises-security/oss-report/v1",
            gate,
            status: "passed",
            findings: vec![],
            uninspected: vec![],
            evidence_sha256: vec![],
            external_actions: false,
        }
    }
    pub(super) fn finding(&mut self, code: &'static str, location: &str) {
        self.findings.push(OssFinding {
            code,
            location: location.into(),
        });
    }
    pub(super) fn gap(&mut self, code: &'static str, location: &str) {
        self.uninspected.push(OssFinding {
            code,
            location: location.into(),
        });
    }
    pub(super) fn finish(mut self) -> Self {
        self.status = if !self.uninspected.is_empty() {
            "incomplete"
        } else if !self.findings.is_empty() {
            "findings"
        } else {
            "passed"
        };
        self
    }
    /// Exit codes match the endpoint CLI: 0 pass, 2 incomplete, 3 findings.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self.status {
            "passed" => 0,
            "findings" => 3,
            _ => 2,
        }
    }
}
