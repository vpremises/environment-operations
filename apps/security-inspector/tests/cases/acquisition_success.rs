#![cfg(target_os = "linux")]

//! Successful acquisition returns only deterministic artifact metadata.

use crate::support::{acquire, daily_report, report_request, TempDirectory};
use std::fs;
use vpremises::{
    DEPARTMENT_DAILY_REPORT_SCHEMA, MOUNTED_SHAREPOINT_RECEIPT_SCHEMA, REPORT_MEDIA_TYPE,
};

#[cfg(target_os = "linux")]
#[test]
fn acquires_one_report_without_disclosing_body_or_path() {
    let root = TempDirectory::create();
    fs::create_dir(root.0.join("daily")).expect("daily directory");
    let source = daily_report("private report body");
    fs::write(root.0.join("daily/report.json"), source.as_bytes()).expect("report");
    let request = report_request(&root.0, "daily/report.json");
    let first = acquire(&request, &root.0).expect("first receipt");
    let second = acquire(&request, &root.0).expect("second receipt");
    assert_eq!(first, second);
    assert_eq!(first.schema, MOUNTED_SHAREPOINT_RECEIPT_SCHEMA);
    assert_eq!(first.schema_id, DEPARTMENT_DAILY_REPORT_SCHEMA);
    assert_eq!(first.media_type, REPORT_MEDIA_TYPE);
    assert_eq!(first.size_bytes, source.len() as u64);
    assert_eq!(first.digest_sha256.len(), 64);
    assert!(!first.external_actions);
    let serialized = serde_json::to_string(&first).expect("serialize receipt");
    for secret in [
        "private report body",
        "daily/report.json",
        root.0.to_str().expect("path"),
    ] {
        assert!(!serialized.contains(secret));
    }
}
