//! Request validation binds expected identity, classification, and byte limits before I/O.

use crate::{
    model::{MountedSharePointReportRequest, ReportAcquisitionError, ReportClassification},
    report::error,
    validation::{valid_date, valid_identifier},
    MAX_REPORT_BYTES, MOUNTED_SHAREPOINT_REQUEST_SCHEMA,
};

pub(super) fn validate(
    request: &MountedSharePointReportRequest,
) -> Result<(), ReportAcquisitionError> {
    if request.schema != MOUNTED_SHAREPOINT_REQUEST_SCHEMA {
        return Err(error::create(
            "vpremises.report.request-schema-unsupported",
            "$.schema",
            "the mounted SharePoint report request schema is unsupported",
        ));
    }
    for (field, value) in identifiers(request) {
        if !valid_identifier(value) {
            return Err(error::create(
                "vpremises.report.identifier-invalid",
                field,
                "the identifier must contain only lowercase ASCII letters, digits, '.', '_' or '-'",
            ));
        }
    }
    if !valid_date(&request.expected_report.reporting_date) {
        return Err(error::create(
            "vpremises.report.reporting-date-invalid",
            "$.expected_report.reporting_date",
            "reporting_date must use YYYY-MM-DD form",
        ));
    }
    if request.expected_report.customer_data
        != (request.expected_report.classification == ReportClassification::CustomerConfidential)
    {
        return Err(error::create(
            "vpremises.report.classification-invalid",
            "$.expected_report.classification",
            "customer_data and customer-confidential classification must match",
        ));
    }
    if !(1..=MAX_REPORT_BYTES).contains(&request.max_bytes) {
        return Err(error::create(
            "vpremises.report.limit-invalid",
            "$.max_bytes",
            "max_bytes must be between 1 and 4194304",
        ));
    }
    Ok(())
}

fn identifiers(request: &MountedSharePointReportRequest) -> [(&'static str, &str); 8] {
    [
        ("$.request_id", request.request_id.as_str()),
        ("$.correlation_id", request.correlation_id.as_str()),
        ("$.artifact_id", request.artifact_id.as_str()),
        (
            "$.expected_report.headquarters_id",
            request.expected_report.headquarters_id.as_str(),
        ),
        (
            "$.expected_report.department_id",
            request.expected_report.department_id.as_str(),
        ),
        (
            "$.expected_report.reporting_team_id",
            request.expected_report.reporting_team_id.as_str(),
        ),
        (
            "$.expected_report.owner_account_id",
            request.expected_report.owner_account_id.as_str(),
        ),
        (
            "$.allowlisted_root.id",
            request.allowlisted_root.id.as_str(),
        ),
    ]
}
