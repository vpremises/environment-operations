//! Bind manifest identity, distributable entry points and applicable legal documents.
use super::{package, OssReport};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) fn check(
    root: &Path,
    kind: &str,
    name: &str,
    version: &str,
    files: &BTreeSet<String>,
    documents: &BTreeMap<String, Vec<u8>>,
    report: &mut OssReport,
) -> Result<(), &'static str> {
    for legal in ["LICENSE", "NOTICE", "README.md"] {
        if !files.contains(legal) {
            report.finding("archive-document-missing", legal);
        }
    }
    for entry in std::fs::read_dir(root).map_err(|_| "legal-inventory-unavailable")? {
        let entry = entry.map_err(|_| "legal-inventory-unavailable")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "legal-path-encoding")?;
        if (name.starts_with("LICENSE")
            || ["THIRD_PARTY_NOTICES.md", "THIRD-PARTY-NOTICES.md"].contains(&name.as_str()))
            && entry
                .file_type()
                .map_err(|_| "legal-inventory-unavailable")?
                .is_file()
            && !files.contains(&name)
        {
            report.finding("archive-applicable-notice-missing", &name);
        }
    }
    let manifest = if kind == "npm" {
        "package.json"
    } else {
        "Cargo.toml"
    };
    let Some(bytes) = documents.get(manifest) else {
        report.finding("archive-manifest-missing", manifest);
        return Ok(());
    };
    let value: serde_json::Value = if kind == "npm" {
        serde_json::from_slice(bytes).map_err(|_| "archive-manifest-invalid")?
    } else {
        let text = std::str::from_utf8(bytes).map_err(|_| "archive-manifest-invalid")?;
        let value: toml::Value = toml::from_str(text).map_err(|_| "archive-manifest-invalid")?;
        serde_json::to_value(value).map_err(|_| "archive-manifest-invalid")?
    };
    super::package_manifest::check(root, kind, &value, report);
    let value = if kind == "npm" {
        &value
    } else {
        &value["package"]
    };
    if value["name"] != name || value["version"] != version {
        report.finding("archive-identity-mismatch", manifest);
    }
    if kind == "npm" {
        for field in ["main", "module", "types", "typings", "exports", "bin"] {
            let mut targets = vec![];
            package::entries(&value[field], &mut targets, 0)?;
            for target in targets {
                let target = target.strip_prefix("./").unwrap_or(&target);
                if !super::paths::relative(target) {
                    report.finding("npm-entry-invalid", field);
                    continue;
                }
                let pattern = globset::Glob::new(target)
                    .map_err(|_| "npm-entry-pattern-invalid")?
                    .compile_matcher();
                if !files.iter().any(|p| pattern.is_match(p)) {
                    report.finding("npm-entry-missing", field);
                }
            }
        }
    }
    Ok(())
}
