//! Portable report tests prevent unsupported checks becoming security passes.

use crate::support::{observer_config, TempDirectory};
use vpremises::security_report;

#[test]
fn complete_metadata_does_not_imply_complete_security_coverage() {
    let root = TempDirectory::create();
    let report = security_report(&observer_config(&root.0), "test-wsl").expect("report");
    assert!(report.observation.ok);
    assert_eq!(report.outcome, "incomplete");
    assert_eq!(
        report
            .checks
            .iter()
            .filter(|item| item.status == "unsupported")
            .count(),
        3
    );
    assert!(report.finished_at_unix_ms >= report.started_at_unix_ms);
    assert_eq!(report.configuration_sha256.len(), 64);
    let json = serde_json::to_string(&report).expect("JSON report");
    assert!(!json.contains(root.0.to_str().expect("path")));
    assert!(!report.external_actions);
}

#[test]
fn invalid_environment_identifiers_are_rejected() {
    let root = TempDirectory::create();
    assert!(security_report(&observer_config(&root.0), "user@host").is_err());
}

#[test]
fn failed_collection_remains_incomplete() {
    let root = TempDirectory::create();
    let mut config = observer_config(&root.0);
    config.limits.max_entries = 0;
    let report = security_report(&config, "test-windows").expect("report");
    assert!(!report.observation.ok);
    assert_eq!(report.checks[0].status, "incomplete");
}
