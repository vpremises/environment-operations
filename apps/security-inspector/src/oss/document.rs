//! Read regular, no-follow JSON/TOML documents without printing parser inputs.
use super::OssReport;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) fn read(path: &Path, maximum: u64) -> Result<Vec<u8>, &'static str> {
    crate::audit::io::read(path, maximum)
}
pub(super) fn load(path: &Path, report: &mut OssReport) -> Result<Value, &'static str> {
    let bytes = read(path, 1_048_576)?;
    report
        .evidence_sha256
        .push(format!("{:x}", Sha256::digest(&bytes)));
    let value = if path.extension().is_some_and(|s| s == "toml")
        || (path.file_name().is_some_and(|s| s == "config")
            && path
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|s| s == ".cargo"))
    {
        let text = std::str::from_utf8(&bytes).map_err(|_| "document-invalid")?;
        let value: toml::Value = toml::from_str(text).map_err(|_| "document-invalid")?;
        serde_json::to_value(value).map_err(|_| "document-invalid")?
    } else {
        serde_json::from_slice(&bytes).map_err(|_| "document-invalid")?
    };
    if !value.is_object() {
        return Err("document-invalid");
    }
    Ok(value)
}

/// Distinguish absence from an inaccessible, linked or unstable required document.
pub(super) fn required(root: &Path, name: &str, missing: &'static str, report: &mut OssReport) {
    let path = root.join(name);
    if std::fs::symlink_metadata(&path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
        report.finding(missing, name);
    } else if let Err(code) = read(&path, 4_194_304) {
        report.gap(code, name);
    }
}
