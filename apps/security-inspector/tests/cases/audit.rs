//! Missing engines, changed pins and stale observations must never yield a pass.
use crate::support::{observer_config, TempDirectory};
use vpremises::{audit, AuditConfig, Collectors, ContentCollector, Tool};

#[test]
fn absent_collectors_remain_incomplete() {
    let root = TempDirectory::create();
    let config = AuditConfig {
        schema: "vpremises-security/audit/v1".into(),
        collector_budget_seconds: 300,
        observer: observer_config(&root.0),
        collectors: Collectors {
            content: None,
            network: None,
            boundary: None,
        },
    };
    let report = audit(&config, "wsl-test").expect("audit");
    assert_eq!(report.outcome, "incomplete");
    assert_eq!(report.schema, "vpremises-security/report/v2");
    assert_eq!(
        report
            .checks
            .iter()
            .filter(|c| c.status == "incomplete")
            .count(),
        3
    );
    assert!(!serde_json::to_string(&report)
        .unwrap()
        .contains(root.0.to_str().unwrap()));
}
#[test]
fn incorrect_executable_pin_stays_incomplete_without_reading_content() {
    let root = TempDirectory::create();
    let config = AuditConfig {
        schema: "vpremises-security/audit/v1".into(),
        collector_budget_seconds: 300,
        observer: observer_config(&root.0),
        collectors: Collectors {
            content: Some(ContentCollector {
                tool: Tool {
                    executable: "/usr/bin/false".into(),
                    sha256: "0".repeat(64),
                    timeout_seconds: 1,
                },
                settings: "/nonexistent".into(),
            }),
            network: None,
            boundary: None,
        },
    };
    let report = audit(&config, "wsl-test").expect("audit");
    assert_eq!(report.checks[1].reason, "tool-pin-mismatch");
    assert_eq!(report.outcome, "incomplete");
}
