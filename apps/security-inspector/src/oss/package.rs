//! Public distribution policy inspects declarations without installing dependencies.
use super::{archive, document, OssReport};
use serde_json::Value;
use std::path::Path;

/// Inspect an npm/Cargo release manifest and optional exact gzip-tar distribution.
/// # Errors
/// Reject invalid roots, kinds, documents or archives without parser input disclosure.
pub fn inspect_package(
    root: &Path,
    kind: &str,
    archive_path: Option<&Path>,
) -> Result<OssReport, &'static str> {
    if !["npm", "cargo"].contains(&kind) {
        return Err("package-kind-invalid");
    }
    let root = super::root(root)?;
    let mut report = OssReport::new("public-package");
    let file = if kind == "npm" {
        "package.json"
    } else {
        "Cargo.toml"
    };
    let value = document::load(&root.join(file), &mut report)?;
    let package = if kind == "npm" {
        &value
    } else {
        &value["package"]
    };
    super::package_manifest::check(&root, kind, &value, &mut report);
    let name = package["name"].as_str().ok_or("package-identity-invalid")?;
    let version = package["version"]
        .as_str()
        .ok_or("package-identity-invalid")?;
    for file in ["LICENSE", "NOTICE", "README.md"] {
        document::required(&root, file, "package-document-missing", &mut report);
    }
    super::catalog::check(&root, &mut report);
    if let Some(path) = archive_path {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|_| "archive-unavailable")?
                .join(path)
        };
        archive::check(&root, &path, kind, name, version, &mut report)?;
        // Structural checks cannot certify absent secrets: require the configured content collector.
        report.gap("archive-content-audit-required", "");
    }
    Ok(report.finish())
}

pub(super) fn entries(
    value: &Value,
    output: &mut Vec<String>,
    depth: usize,
) -> Result<(), &'static str> {
    if depth > 64 || output.len() > 15_000 {
        return Err("npm-entry-budget");
    }
    match value {
        Value::String(s) => output.push(s.clone()),
        Value::Object(o) => {
            for v in o.values() {
                entries(v, output, depth + 1)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                entries(v, output, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}
