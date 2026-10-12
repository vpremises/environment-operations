//! Bundle setup must preserve external configuration boundaries and existing files.
use crate::support::TempDirectory;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, os::unix::fs::PermissionsExt};
use vpremises::initialize_bundle;

#[test]
fn setup_pins_bundle_and_creates_private_unconfigured_settings() {
    let temp = TempDirectory::create();
    let bundle = temp.0.join("bundle");
    fs::create_dir(&bundle).unwrap();
    fs::create_dir(bundle.join("tools")).unwrap();
    let root = temp.0.join("root");
    fs::create_dir(&root).unwrap();
    let mut tools = serde_json::Map::new();
    for name in [
        "zixcel-repository-security",
        "crowsi-host-network-sensor",
        "crowsi-boundary-monitor",
        "gitleaks",
    ] {
        let bytes = b"\x7fELFsynthetic-non-executable";
        fs::write(bundle.join("tools").join(name), bytes).unwrap();
        tools.insert(
            name.into(),
            json!({"sha256":format!("{:x}", Sha256::digest(bytes))}),
        );
    }
    fs::write(
        bundle.join("collectors.json"),
        serde_json::to_vec(&json!({
        "schema":"vpremises-security/collectors/v1", "tools":tools}))
        .unwrap(),
    )
    .unwrap();
    let output = temp.0.join("private");
    initialize_bundle(&bundle, &output, &root).unwrap();
    assert_eq!(
        fs::metadata(&output).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(output.join("audit.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let config = vpremises::load_audit_config(&output.join("audit.json")).unwrap();
    assert!(config.collectors.boundary.is_none());
    assert!(config.collectors.network.unwrap().baseline.is_none());
    assert!(initialize_bundle(&bundle, &output, &root).is_err());
    assert!(initialize_bundle(&bundle, &root.join("inside"), &root).is_err());
    fs::write(bundle.join("tools/gitleaks"), b"changed").unwrap();
    assert!(initialize_bundle(&bundle, &temp.0.join("another"), &root).is_err());
    assert!(!temp.0.join("another").exists());
}
