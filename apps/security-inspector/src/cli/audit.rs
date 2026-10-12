//! Configured local collectors return coverage gaps instead of false success.
use serde_json::{json, Value};
use std::path::Path;
pub(super) fn run(path: &Path, environment: &str) -> Result<Value, (u8, Value)> {
    let config = vpremises::load_audit_config(path).map_err(invalid)?;
    let report = vpremises::audit(&config, environment).map_err(invalid)?;
    let code = match report.outcome {
        "passed" => 0,
        "findings" => 3,
        _ => 2,
    };
    let value = serde_json::to_value(report).expect("audit report serializes");
    if code == 0 {
        Ok(value)
    } else {
        Err((code, value))
    }
}
fn invalid(reason: &'static str) -> (u8, Value) {
    (
        1,
        json!({"ok":false,"code":"audit-invalid","reason":reason,"external_actions":false}),
    )
}

/// Resolve the bundle beside this executable; never discover system collectors.
pub(super) fn initialize(directory: &Path, root: &Path) -> Result<Value, (u8, Value)> {
    let executable = std::env::current_exe().map_err(|_| invalid("bundle-unavailable"))?;
    let bundle = executable
        .parent()
        .ok_or_else(|| invalid("bundle-unavailable"))?;
    vpremises::initialize_bundle(bundle, directory, root).map_err(invalid)?;
    Ok(
        json!({"ok":true,"configuration_created":true,"security_verdict":"not-run",
        "remaining_evidence":["approved-network-baseline","fresh-boundary-observation"],"external_actions":false}),
    )
}
