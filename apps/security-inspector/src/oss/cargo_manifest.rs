//! Cargo publication rejects private sources and inherited local dependencies.
use super::{document, OssReport};
use serde_json::Value;
use std::path::Path;

pub(super) fn cargo(root: &Path, value: &Value, report: &mut OssReport) {
    if value["package"]["publish"] != serde_json::json!(["crates-io"]) {
        report.finding("cargo-publication-restriction", "package/publish");
    }
    let rust = value["package"]["rust-version"]
        .as_str()
        .unwrap_or_default();
    if !(2..=3).contains(&rust.split('.').count())
        || rust
            .split('.')
            .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
    {
        report.finding("cargo-numeric-toolchain-required", "package/rust-version");
    }
    let mut sections = vec![value, &value["workspace"]];
    if let Some(targets) = value["target"].as_object() {
        sections.extend(targets.values());
    }
    for section in sections {
        for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(raw) = section.get(key) {
                let Some(deps) = raw.as_object() else {
                    report.gap("dependency-section-invalid", key);
                    continue;
                };
                for (name, dep) in deps {
                    if ["path", "git", "workspace"]
                        .iter()
                        .any(|k| dep.get(k).is_some())
                    {
                        report.finding("cargo-nonregistry-dependency", &format!("{key}/{name}"));
                    }
                    if dep.get("registry").is_some_and(|r| r != "crates-io") {
                        report.finding("cargo-alternate-registry", &format!("{key}/{name}"));
                    }
                }
            }
        }
    }
    if value.get("patch").is_some() || value.get("replace").is_some() {
        report.finding("cargo-dependency-replacement", "");
    }
    for name in [".cargo/config.toml", ".cargo/config"] {
        let config = root.join(name);
        if config.try_exists().unwrap_or(true) {
            match document::load(&config, report) {
                Ok(v) if v.get("source").is_some() || v.get("registries").is_some() => {
                    report.finding("cargo-registry-configuration", name);
                }
                Err(code) => report.gap(code, name),
                _ => {}
            }
        }
    }
}
