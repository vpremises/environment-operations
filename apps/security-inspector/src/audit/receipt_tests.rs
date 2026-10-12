//! Adversarial engine counters cannot erase findings or manufacture clean coverage.
use super::super::{boundary_receipt, content, content_receipt};
use serde_json::json;

fn content_value() -> serde_json::Value {
    json!({"schema":"zixcel://repository-security/receipt/v1", "scope":"directory-secret-disclosure/v1",
        "engines":{"gitleaks":{"status":"completed"}},"status":"findings",
        "open_findings":1,"reviewed_findings":0,"findings":[{"fingerprint":"synthetic"}],"uninspected":[]})
}
#[test]
fn content_receipt_rejects_hidden_and_reviewed_findings() {
    let mut value = content_value();
    assert_eq!(content_receipt::validate(1, &value).unwrap(), (1, 0));
    value["open_findings"] = json!(0);
    value["status"] = json!("passed");
    assert!(content_receipt::validate(0, &value).is_err());
    value["reviewed_findings"] = json!(1);
    assert!(content_receipt::validate(0, &value).is_err());
}
#[test]
fn later_root_failure_preserves_earlier_findings_and_continues() {
    let results = vec![
        Ok(content_value()),
        Err("collector-invalid-output"),
        Ok(content_value()),
    ];
    let report = content::summarize(results.into_iter()).unwrap();
    assert_eq!(report.status, "incomplete");
    assert_eq!(report.finding_count, 2);
    assert_eq!(report.gap_count, 1);
    assert!(report.evidence_sha256.is_some());
}
fn boundary_value() -> serde_json::Value {
    json!({"schema":"crowsi://network/boundary-snapshot/v1","external_actions":false,"generated_at":"synthetic",
        "overall_status":"healthy","summary":{"environment_count":1,"healthy_count":1,"attention_count":0,"unknown_count":0},
        "environments":[{"id":"synthetic","status":"healthy","public_ingress":false}]})
}
#[test]
fn boundary_counter_overflow_and_hidden_unknown_are_rejected() {
    let input = json!({"generated_at":"synthetic","environments":[{"id":"synthetic"}]});
    let mut v = boundary_value();
    assert_eq!(boundary_receipt::validate(&v, &input).unwrap(), (0, 0));
    v["summary"]["attention_count"] = json!(u64::MAX);
    assert!(boundary_receipt::validate(&v, &input).is_err());
    v = boundary_value();
    v["environments"][0]["status"] = json!("unknown");
    assert!(boundary_receipt::validate(&v, &input).is_err());
}
#[test]
fn boundary_identity_time_and_ingress_are_bound_to_evidence() {
    let input = json!({"generated_at":"synthetic","environments":[{"id":"synthetic"}]});
    let mut v = boundary_value();
    v["environments"][0]["public_ingress"] = json!(true);
    assert_eq!(boundary_receipt::validate(&v, &input).unwrap(), (1, 0));
    v["generated_at"] = json!("different");
    assert!(boundary_receipt::validate(&v, &input).is_err());
    v = boundary_value();
    v["environments"][0]["id"] = json!("different");
    assert!(boundary_receipt::validate(&v, &input).is_err());
}
