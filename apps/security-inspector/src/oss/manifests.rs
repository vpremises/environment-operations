//! Validate declared license metadata without interpreting package code.
use super::{document, OssPolicy, OssReport};
use std::path::Path;

pub(super) fn check(root: &Path, policy: &OssPolicy, report: &mut OssReport) {
    for name in ["package.json", "Cargo.toml", "pyproject.toml"] {
        let path = root.join(name);
        if !path.try_exists().unwrap_or(true) {
            continue;
        }
        let value = match document::load(&path, report) {
            Ok(value) => value,
            Err(code) => {
                report.gap(code, name);
                continue;
            }
        };
        let package = if name == "package.json" {
            &value
        } else {
            value
                .get("package")
                .or_else(|| value.get("project"))
                .unwrap_or(&serde_json::Value::Null)
        };
        if !package.is_object()
            && (value.get("package").is_some() || value.get("project").is_some())
        {
            report.gap("manifest-package-invalid", name);
            continue;
        }
        if name != "package.json" && package.get("name").is_none() {
            continue;
        }
        let license = package
            .get("license")
            .and_then(|v| v.as_str().or_else(|| v.get("text")?.as_str()));
        if license != Some(&policy.license) {
            report.finding("manifest-license-mismatch", name);
        }
    }
}
