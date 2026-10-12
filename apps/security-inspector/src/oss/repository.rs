//! Community policy checks operate on one explicitly selected Git checkout.
use super::{document, git, manifests, paths, source, OssReport};
use sha2::{Digest, Sha256};
use std::path::Path;

const REQUIRED: [&str; 7] = [
    "LICENSE",
    "NOTICE",
    "CONTRIBUTING.md",
    "CODE_OF_CONDUCT.md",
    "SECURITY.md",
    ".github/pull_request_template.md",
    ".github/ISSUE_TEMPLATE/config.yml",
];

/// Inspect community files, root licenses, tracked artifacts and optional Rust layout.
/// # Errors
/// Invalid root or policy is returned as a stable code without input values.
pub fn inspect_repository(
    root: &Path,
    policy_path: Option<&Path>,
) -> Result<OssReport, &'static str> {
    let root = super::root(root)?;
    git::validate_root(&root)?;
    let policy = super::policy(&root, policy_path)?;
    let mut report = OssReport::new("repository");
    report.evidence_sha256.push(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&policy).map_err(|_| "policy-invalid")?)
    ));
    for name in REQUIRED {
        document::required(&root, name, "required-community-file-missing", &mut report);
    }
    match git::names(&root, &["ls-files", "-z"]) {
        Ok(names) => {
            report
                .evidence_sha256
                .push(format!("{:x}", Sha256::digest(names.join("\0"))));
            for name in names {
                if paths::forbidden(&name, &policy) {
                    report.finding("tracked-private-or-generated-path", &name);
                }
            }
        }
        Err(code) => report.gap(code, ""),
    }
    manifests::check(&root, &policy, &mut report);
    if let Some(settings) = &policy.rust_source {
        source::check(&root, settings, &mut report);
    }
    Ok(report.finish())
}
