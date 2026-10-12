//! Intrinsic validation proves report identity, lineage, bounds, and classification.

use crate::{
    model::{DepartmentDailyReport, ReportAcquisitionError, ReportClassification},
    report::error,
    validation::{unique, valid_date, valid_identifier, valid_timestamp},
    DEPARTMENT_DAILY_REPORT_SCHEMA,
};

pub(super) fn validate(bytes: &[u8]) -> Result<DepartmentDailyReport, ReportAcquisitionError> {
    let report: DepartmentDailyReport = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    if report.schema != DEPARTMENT_DAILY_REPORT_SCHEMA
        || !timestamps_valid(&report)
        || !identifiers_valid(&report)
        || !lineage_present(&report)
        || !metrics_bounded(&report)
        || !exceptions_valid(&report)
        || (report.customer_data
            != (report.classification == ReportClassification::CustomerConfidential))
        || report.summary.trim().is_empty()
        || report.summary.chars().count() > 1_000
    {
        return Err(invalid());
    }
    Ok(report)
}

fn identifiers_valid(report: &DepartmentDailyReport) -> bool {
    let fields = [
        report.operational_report_id.as_str(),
        report.headquarters_id.as_str(),
        report.department_id.as_str(),
        report.reporting_team_id.as_str(),
        report.owner_account_id.as_str(),
    ];
    let lists = [
        report.source_case_ids.as_slice(),
        report.source_event_ids.as_slice(),
        report.source_task_ids.as_slice(),
        report.source_report_ids.as_slice(),
        report.source_artifact_ids.as_slice(),
        report.escalated_decision_ids.as_slice(),
    ];
    fields.iter().all(|value| valid_identifier(value))
        && lists.iter().all(|values| {
            values.len() <= 1_024
                && values.iter().all(|value| valid_identifier(value))
                && unique(values)
        })
}

fn lineage_present(report: &DepartmentDailyReport) -> bool {
    !report.source_task_ids.is_empty()
        && !report.source_report_ids.is_empty()
        && !report.source_artifact_ids.is_empty()
}

fn timestamps_valid(report: &DepartmentDailyReport) -> bool {
    valid_date(&report.reporting_date)
        && valid_timestamp(&report.window_started_at)
        && valid_timestamp(&report.window_ended_at)
        && valid_timestamp(&report.generated_at)
}

fn metrics_bounded(report: &DepartmentDailyReport) -> bool {
    [
        report.metrics.completed_tasks,
        report.metrics.open_tasks,
        report.metrics.blocked_tasks,
        report.metrics.reported_deliverables,
        report.metrics.open_escalations,
    ]
    .iter()
    .all(|value| *value <= 1_000_000_000)
}

fn exceptions_valid(report: &DepartmentDailyReport) -> bool {
    report.exceptions.len() <= 1_024
        && report.exceptions.iter().all(|exception| {
            valid_identifier(&exception.case_id)
                && matches!(
                    exception.severity.as_str(),
                    "low" | "medium" | "high" | "critical"
                )
                && !exception.status.trim().is_empty()
                && exception.status.len() <= 64
                && exception
                    .decision_id
                    .as_deref()
                    .is_none_or(valid_identifier)
        })
}

fn invalid() -> ReportAcquisitionError {
    error::create(
        "vpremises.report.document-invalid",
        "$.relative_file",
        "the report violates the closed department daily report contract",
    )
}
