//! Contract tests bind expected context and keep request/report objects closed.

#[cfg(target_os = "linux")]
use crate::support::{acquire, daily_report, report_request, TempDirectory};
#[cfg(target_os = "linux")]
use std::fs;
use vpremises::{
    MountedSharePointReportRequest, MOUNTED_SHAREPOINT_RECEIPT_SCHEMA,
    MOUNTED_SHAREPOINT_REQUEST_SCHEMA,
};
#[cfg(target_os = "linux")]
use vpremises::{ReportClassification, ReportRecordMode};

#[test]
fn request_rejects_unknown_fields_and_schemas_are_pinned() {
    let source = include_str!("../../examples/mounted-sharepoint-report.request.json").replacen(
        "\n}",
        ",\n  \"unexpected\": true\n}",
        1,
    );
    assert!(serde_json::from_str::<MountedSharePointReportRequest>(&source).is_err());
    for (source, expected_id) in [
        (
            include_str!(
                "../../schemas/vpremises.mounted-sharepoint-report-request.v1.schema.json"
            ),
            MOUNTED_SHAREPOINT_REQUEST_SCHEMA,
        ),
        (
            include_str!(
                "../../schemas/vpremises.mounted-sharepoint-report-receipt.v1.schema.json"
            ),
            MOUNTED_SHAREPOINT_RECEIPT_SCHEMA,
        ),
    ] {
        let schema: serde_json::Value = serde_json::from_str(source).expect("schema JSON");
        assert_eq!(schema["$id"], expected_id);
        assert_eq!(schema["additionalProperties"], false);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_every_expected_context_mismatch() {
    let root = TempDirectory::create();
    fs::write(root.0.join("report.json"), daily_report("bounded")).expect("report");
    let baseline = report_request(&root.0, "report.json");
    let mut mismatches = Vec::new();
    let mut request = baseline.clone();
    request.expected_report.record_mode = ReportRecordMode::Runtime;
    mismatches.push(("$.expected_report.record_mode", request));
    let mut request = baseline.clone();
    request.expected_report.reporting_date = "2026-07-25".to_owned();
    mismatches.push(("$.expected_report.reporting_date", request));
    let mut request = baseline.clone();
    request.expected_report.headquarters_id = "another-headquarters".to_owned();
    mismatches.push(("$.expected_report.headquarters_id", request));
    let mut request = baseline.clone();
    request.expected_report.department_id = "another-department".to_owned();
    mismatches.push(("$.expected_report.department_id", request));
    let mut request = baseline.clone();
    request.expected_report.reporting_team_id = "another-team".to_owned();
    mismatches.push(("$.expected_report.reporting_team_id", request));
    let mut request = baseline.clone();
    request.expected_report.owner_account_id = "another-owner".to_owned();
    mismatches.push(("$.expected_report.owner_account_id", request));
    let mut request = baseline;
    request.expected_report.classification = ReportClassification::RestrictedSensitive;
    mismatches.push(("$.expected_report.classification", request));
    for (field, request) in mismatches {
        let error = acquire(&request, &root.0).expect_err("context mismatch");
        assert_eq!(error.code, "vpremises.report.expected-context-mismatch");
        assert_eq!(error.field, field);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_unknown_or_wrong_report_documents() {
    let root = TempDirectory::create();
    let unknown = daily_report("unknown").replacen("\n}", ",\n  \"unexpected\": true\n}", 1);
    fs::write(root.0.join("unknown.json"), unknown).expect("unknown report");
    fs::write(
        root.0.join("wrong.json"),
        r#"{"schema":"estate://operations/another-report/v1"}"#,
    )
    .expect("wrong report");
    for name in ["unknown.json", "wrong.json"] {
        let error = acquire(&report_request(&root.0, name), &root.0)
            .expect_err("closed report must reject");
        assert_eq!(error.code, "vpremises.report.document-invalid");
    }
}
