//! Expected context prevents a valid report being accepted for the wrong operational case.

use crate::{
    model::{DepartmentDailyReport, ExpectedDepartmentDailyReport, ReportAcquisitionError},
    report::error,
};

pub(super) fn validate(
    expected: &ExpectedDepartmentDailyReport,
    report: &DepartmentDailyReport,
) -> Result<(), ReportAcquisitionError> {
    let mismatches = [
        (
            expected.record_mode != report.record_mode,
            "$.expected_report.record_mode",
        ),
        (
            expected.reporting_date != report.reporting_date,
            "$.expected_report.reporting_date",
        ),
        (
            expected.headquarters_id != report.headquarters_id,
            "$.expected_report.headquarters_id",
        ),
        (
            expected.department_id != report.department_id,
            "$.expected_report.department_id",
        ),
        (
            expected.reporting_team_id != report.reporting_team_id,
            "$.expected_report.reporting_team_id",
        ),
        (
            expected.owner_account_id != report.owner_account_id,
            "$.expected_report.owner_account_id",
        ),
        (
            expected.classification != report.classification,
            "$.expected_report.classification",
        ),
        (
            expected.customer_data != report.customer_data,
            "$.expected_report.customer_data",
        ),
    ];
    if let Some((_, field)) = mismatches.into_iter().find(|(mismatch, _)| *mismatch) {
        return Err(error::create(
            "vpremises.report.expected-context-mismatch",
            field,
            "the report does not match its expected acquisition context",
        ));
    }
    Ok(())
}
