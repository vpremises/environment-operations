//! The acquired body is private and closed; only validated metadata reaches the receipt.

use super::{ReportClassification, ReportRecordMode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DepartmentDailyReport {
    /// Versioned contract URI.
    pub schema: String,
    /// Operational report id.
    pub operational_report_id: String,
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
    /// Window started at.
    pub window_started_at: String,
    /// Window ended at.
    pub window_ended_at: String,
    /// Generated at.
    pub generated_at: String,
    /// Source case ids.
    pub source_case_ids: Vec<String>,
    /// Source event ids.
    pub source_event_ids: Vec<String>,
    /// Source task ids.
    pub source_task_ids: Vec<String>,
    /// Source report ids.
    pub source_report_ids: Vec<String>,
    /// Source artifact ids.
    pub source_artifact_ids: Vec<String>,
    /// Escalated decision ids.
    pub escalated_decision_ids: Vec<String>,
    /// Metrics.
    pub metrics: DepartmentDailyMetrics,
    /// Exceptions.
    pub exceptions: Vec<DepartmentDailyException>,
    /// Classification.
    pub classification: ReportClassification,
    /// Customer data.
    pub customer_data: bool,
    /// Summary.
    pub summary: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DepartmentDailyMetrics {
    /// Completed tasks.
    pub completed_tasks: u64,
    /// Open tasks.
    pub open_tasks: u64,
    /// Blocked tasks.
    pub blocked_tasks: u64,
    /// Reported deliverables.
    pub reported_deliverables: u64,
    /// Open escalations.
    pub open_escalations: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DepartmentDailyException {
    /// Case id.
    pub case_id: String,
    /// Severity.
    pub severity: String,
    /// Status.
    pub status: String,
    /// Decision id.
    pub decision_id: Option<String>,
}
