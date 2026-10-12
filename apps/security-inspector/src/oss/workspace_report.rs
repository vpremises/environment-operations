//! Workspace receipts preserve individual gate outcomes.
use super::OssReport;
use serde::Serialize;

/// A repository-scoped result preserves gaps without terminating independent checks.
#[derive(Debug, Serialize)]
pub struct RepositoryResult {
    /// Canonical owner/repository relative location.
    pub repository: String,
    /// Selected policy gate receipt.
    pub report: OssReport,
}

/// Current canonical workspace receipt.
#[derive(Debug, Serialize)]
pub struct WorkspaceReport {
    /// Stable workspace contract.
    pub schema: &'static str,
    /// Combined gate result, with incomplete taking precedence.
    pub status: &'static str,
    /// Each physically present repository, including failures to inspect.
    pub repositories: Vec<RepositoryResult>,
    /// Never fetch, publish or install while inspecting.
    pub external_actions: bool,
}
impl WorkspaceReport {
    /// Preserve common CLI exit semantics.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self.status {
            "passed" => 0,
            "findings" => 3,
            _ => 2,
        }
    }
}
