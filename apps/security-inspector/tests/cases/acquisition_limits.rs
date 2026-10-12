#![cfg(target_os = "linux")]

//! Semantic limits reject plausible-looking but unsafe report documents.

use crate::support::{acquire, daily_report, report_request, TempDirectory};
use serde_json::{json, Value};
use std::fs;
use vpremises::ReportClassification;

#[cfg(target_os = "linux")]
#[test]
fn customer_classification_and_flag_must_match_in_request_and_body() {
    let root = TempDirectory::create();
    fs::write(root.0.join("report.json"), daily_report("bounded")).expect("report");
    let mut request = report_request(&root.0, "report.json");
    request.expected_report.classification = ReportClassification::CustomerConfidential;
    assert_eq!(
        acquire(&request, &root.0)
            .expect_err("request classification mismatch")
            .code,
        "vpremises.report.classification-invalid"
    );

    let body = daily_report("bounded").replace(
        "\"classification\": \"internal-confidential\"",
        "\"classification\": \"customer-confidential\"",
    );
    fs::write(root.0.join("report.json"), body).expect("report");
    let error = acquire(&report_request(&root.0, "report.json"), &root.0)
        .expect_err("body classification mismatch");
    assert_eq!(error.code, "vpremises.report.document-invalid");
}

#[cfg(target_os = "linux")]
#[test]
fn invalid_calendar_time_and_unbounded_lineage_are_rejected() {
    let root = TempDirectory::create();
    let invalid_time =
        daily_report("bounded").replace("2026-07-24T17:00:00+09:00", "2026-07-24T24:00:00Z");
    fs::write(root.0.join("report.json"), invalid_time).expect("report");
    let request = report_request(&root.0, "report.json");
    assert_eq!(
        acquire(&request, &root.0).expect_err("invalid time").code,
        "vpremises.report.document-invalid"
    );

    let mut report: Value = serde_json::from_str(&daily_report("bounded")).expect("fixture");
    report["source_case_ids"] = json!((0..1_025)
        .map(|index| format!("case-{index}"))
        .collect::<Vec<_>>());
    fs::write(
        root.0.join("report.json"),
        serde_json::to_vec(&report).expect("report JSON"),
    )
    .expect("report");
    let mut request = report_request(&root.0, "report.json");
    request.max_bytes = 4 * 1024 * 1024;
    assert_eq!(
        acquire(&request, &root.0)
            .expect_err("unbounded lineage")
            .code,
        "vpremises.report.document-invalid"
    );
}
