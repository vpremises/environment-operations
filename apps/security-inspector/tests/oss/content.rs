//! Archive content completion requires exact bytes, scope, native policy and a valid collector.
use super::support::{archive, package, write};
use vpremises::oss::{audit_archive_content, inspect_package};

fn config(root: &std::path::Path, selected: &std::path::Path, content: bool) -> std::path::PathBuf {
    let value = serde_json::json!({
        "schema":"vpremises-security/audit/v1", "collector_budget_seconds":1,
        "observer":{"schema_version":"vpremises.observer/v1",
            "roots":[{"id":"fixture","path":selected}],
            "limits":{"max_depth":8,"max_entries":100,"max_total_bytes":1_000_000},
            "policy":{"metadata_only":true,"follow_symlinks":false}},
        "collectors":{"content":if content { serde_json::json!({"tool":{
            "executable":root.join("missing-engine"),"sha256":"0".repeat(64),"timeout_seconds":1},
            "settings":root.join("detection.json")}) } else { serde_json::Value::Null },
            "network":null,"boundary":null}});
    write(root, "audit.json", value.to_string());
    root.join("audit.json")
}
#[test]
fn missing_collector_and_wrong_root_remain_incomplete() {
    let root = package();
    let settings = tempfile::tempdir().unwrap();
    let path = archive(root.path(), &[("package/LICENSE", b"public")]);
    let mut report = inspect_package(root.path(), "npm", Some(&path)).unwrap();
    let cfg = config(settings.path(), root.path(), false);
    assert_eq!(
        audit_archive_content(&mut report, &path, &cfg).unwrap_err(),
        "archive-content-scope-missing"
    );
    let cfg = config(settings.path(), settings.path(), true);
    assert_eq!(
        audit_archive_content(&mut report, &path, &cfg).unwrap_err(),
        "archive-content-scope-missing"
    );
    assert_eq!(report.exit_code(), 2);
}
#[test]
fn absent_or_modified_native_profile_is_rejected() {
    let root = package();
    let settings = tempfile::tempdir().unwrap();
    let path = archive(root.path(), &[("package/LICENSE", b"public")]);
    let mut report = inspect_package(root.path(), "npm", Some(&path)).unwrap();
    let cfg = config(settings.path(), root.path(), true);
    write(settings.path(), "detection.json", r#"{"gitleaks":{}}"#);
    assert_eq!(
        audit_archive_content(&mut report, &path, &cfg).unwrap_err(),
        "archive-content-profile-required"
    );
    write(
        settings.path(),
        "detection.json",
        r#"{"gitleaks":{"config_file":"native.toml"}}"#,
    );
    write(settings.path(), "native.toml", "title='altered'\n");
    assert_eq!(
        audit_archive_content(&mut report, &path, &cfg).unwrap_err(),
        "archive-content-profile-mismatch"
    );
}
#[test]
fn failed_collector_never_clears_content_gap() {
    let root = package();
    let settings = tempfile::tempdir().unwrap();
    let path = archive(root.path(), &[("package/LICENSE", b"public")]);
    let mut report = inspect_package(root.path(), "npm", Some(&path)).unwrap();
    let cfg = config(settings.path(), root.path(), true);
    write(
        settings.path(),
        "detection.json",
        r#"{"gitleaks":{"config_file":"native.toml"}}"#,
    );
    write(
        settings.path(),
        "native.toml",
        include_bytes!("../../examples/oss.gitleaks.toml"),
    );
    assert_eq!(
        audit_archive_content(&mut report, &path, &cfg).unwrap_err(),
        "archive-content-incomplete"
    );
    assert_eq!(report.exit_code(), 2);
}
#[test]
fn changed_archive_cannot_use_an_earlier_structural_receipt() {
    let root = package();
    let settings = tempfile::tempdir().unwrap();
    let path = archive(root.path(), &[("package/LICENSE", b"public")]);
    let mut report = inspect_package(root.path(), "npm", Some(&path)).unwrap();
    std::fs::write(&path, "changed").unwrap();
    let cfg = config(settings.path(), root.path(), false);
    assert_eq!(
        audit_archive_content(&mut report, &path, &cfg).unwrap_err(),
        "archive-changed"
    );
}
