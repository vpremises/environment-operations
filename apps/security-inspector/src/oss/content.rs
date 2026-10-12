//! Content detection stays in the pinned Gitleaks adapter, never a second regex engine.
use super::{document, OssReport};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Complete the exact-archive content check with an explicit local audit configuration.
/// # Errors
/// Reject missing collectors, uncovered paths, changed bytes or unsuccessful collection.
pub fn audit_archive_content(
    report: &mut OssReport,
    archive: &Path,
    config: &Path,
) -> Result<(), &'static str> {
    let archive = if archive.is_absolute() {
        archive.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| "archive-unavailable")?
            .join(archive)
    };
    let before = hash(&archive)?;
    if !report.evidence_sha256.contains(&before) {
        return Err("archive-changed");
    }
    let mut config = crate::load_audit_config(config)?;
    if config.collectors.content.is_none()
        || !config
            .observer
            .roots
            .iter()
            .any(|r| archive.starts_with(&r.path))
    {
        return Err("archive-content-scope-missing");
    }
    super::content_profile::validate(
        &config
            .collectors
            .content
            .as_ref()
            .ok_or("archive-content-missing")?
            .settings,
        report,
    )?;
    // Scan a private exact-byte copy, not unrelated neighbors. TAR naming also
    // lets the existing archive adapter inspect Cargo's .crate gzip-TAR format.
    let scratch = crate::audit::io::Scratch::create()?;
    let copy = scratch.0.join("candidate.tar.gz");
    std::fs::write(&copy, document::read(&archive, 64 * 1024 * 1024)?)
        .map_err(|_| "archive-copy-failed")?;
    if before != hash(&copy)? {
        return Err("archive-changed");
    }
    config.observer.roots.truncate(1);
    config.observer.roots[0].path.clone_from(&scratch.0);
    let content = crate::audit::oss_content::collect(&config)?;
    if !["completed", "findings"].contains(&content.status) || content.gap_count > 0 {
        return Err("archive-content-incomplete");
    }
    if before != hash(&archive)? {
        return Err("archive-changed");
    }
    report
        .uninspected
        .retain(|g| g.code != "archive-content-audit-required");
    if content.finding_count > 0 {
        report.finding("archive-content-findings", "");
    }
    report.evidence_sha256.push(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&content).map_err(|_| "content-receipt-invalid")?)
    ));
    report.status = if !report.uninspected.is_empty() {
        "incomplete"
    } else if !report.findings.is_empty() {
        "findings"
    } else {
        "passed"
    };
    Ok(())
}
fn hash(path: &Path) -> Result<String, &'static str> {
    Ok(format!(
        "{:x}",
        Sha256::digest(document::read(path, 64 * 1024 * 1024)?)
    ))
}
