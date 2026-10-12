//! Manifest gates prohibit accidental private and local publication dependencies.
use super::support::{has, manifest, package, write};
use vpremises::oss::inspect_package;

#[test]
fn public_npm_manifest_passes() {
    let root = package();
    assert_eq!(
        inspect_package(root.path(), "npm", None)
            .unwrap()
            .exit_code(),
        0
    );
}
#[test]
fn private_and_nonregistry_dependencies_are_rejected() {
    let root = package();
    let mut value = manifest();
    value["private"] = true.into();
    value["dependencies"] = serde_json::json!({"internal":"file:../private"});
    write(root.path(), "package.json", value.to_string());
    let report = inspect_package(root.path(), "npm", None).unwrap();
    assert!(has(&report, "npm-private-package"));
    assert!(has(&report, "npm-nonregistry-dependency"));
    assert!(!serde_json::to_string(&report)
        .unwrap()
        .contains("../private"));
}
#[test]
fn catalog_must_equal_public_allowlist() {
    let root = package();
    write(
        root.path(),
        "public-package-policy.json",
        r#"{"repository_ids":["public"]}"#,
    );
    write(
        root.path(),
        "catalog-v2.json",
        r#"{"entries":[{"repository_id":"private"}]}"#,
    );
    assert!(has(
        &inspect_package(root.path(), "npm", None).unwrap(),
        "catalog-outside-public-allowlist"
    ));
}
#[test]
fn cargo_rejects_local_private_and_overridden_sources() {
    let root = package();
    write(
        root.path(),
        "Cargo.toml",
        r#"
[package]
name = "example"
version = "0.1.0"
license = "Apache-2.0"
repository = "https://github.com/example/library"
rust-version = "1.85"
publish = ["crates-io"]
[dependencies]
internal = { version = "0.1", registry = "private", path = "../internal" }
[target.'cfg(unix)'.dependencies]
local = { workspace = true }
"#,
    );
    write(
        root.path(),
        ".cargo/config.toml",
        "[source.crates-io]\nreplace-with='private'\n",
    );
    let report = inspect_package(root.path(), "cargo", None).unwrap();
    assert!(has(&report, "cargo-nonregistry-dependency"));
    assert!(has(&report, "cargo-alternate-registry"));
    assert!(has(&report, "cargo-registry-configuration"));
}
#[test]
fn malformed_dependency_table_is_incomplete() {
    let root = package();
    let mut value = manifest();
    value["dependencies"] = serde_json::json!(["invalid"]);
    write(root.path(), "package.json", value.to_string());
    assert_eq!(
        inspect_package(root.path(), "npm", None)
            .unwrap()
            .exit_code(),
        2
    );
}

#[test]
fn null_license_cannot_pass_as_declared_ownership() {
    let root = package();
    let mut value = manifest();
    value["license"] = serde_json::Value::Null;
    write(root.path(), "package.json", value.to_string());
    assert!(has(
        &inspect_package(root.path(), "npm", None).unwrap(),
        "package-license-missing"
    ));
}

#[test]
fn legacy_cargo_config_cannot_hide_a_private_source() {
    let root = package();
    write(root.path(), "Cargo.toml", "[package]\nname='fixture'\nversion='0.1.0'\nlicense='MIT'\nrepository='https://github.com/example/fixture'\nrust-version='1.85'\npublish=['crates-io']\n");
    write(
        root.path(),
        ".cargo/config",
        "[registries.internal]\nindex='private-example'\n",
    );
    assert!(has(
        &inspect_package(root.path(), "cargo", None).unwrap(),
        "cargo-registry-configuration"
    ));
}
