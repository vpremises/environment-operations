//! Connect mounted-root content inspection to Zixcel's Gitleaks adapter.
use super::{
    checks::check,
    config::ContentCollector,
    io::{self, Scratch},
    runner,
};
use crate::{CheckCoverage, ObserverConfig};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{ffi::OsStr, path::Path};

pub(super) fn collect(
    config: &ContentCollector,
    observer: &ObserverConfig,
    budget: &super::budget::Budget,
) -> Result<CheckCoverage, &'static str> {
    summarize(
        observer
            .roots
            .iter()
            .map(|root| collect_root(config, &root.path, budget)),
    )
}

fn collect_root(
    config: &ContentCollector,
    root: &Path,
    budget: &super::budget::Budget,
) -> Result<serde_json::Value, &'static str> {
    let tool = budget.tool(&config.tool)?;
    let scratch = Scratch::create()?;
    let receipt = scratch.0.join("receipt.json");
    let (code, summary) = runner::run(
        &tool,
        &[
            OsStr::new("scan-directory"),
            OsStr::new("--root"),
            root.as_os_str(),
            OsStr::new("--config"),
            config.settings.as_os_str(),
            OsStr::new("--output"),
            receipt.as_os_str(),
        ],
        None,
        &scratch,
    )?;
    let bytes = io::read(&receipt, 16 * 1024 * 1024)?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "collector-invalid-output")?;
    super::content_receipt::validate(code, &value)?;
    if summary["passed"].as_bool() != Some(value["status"] == "passed") {
        return Err("collector-invalid-output");
    }
    Ok(value)
}

// Preserve accepted earlier evidence when a later selected root fails.
pub(super) fn summarize(
    results: impl Iterator<Item = Result<serde_json::Value, &'static str>>,
) -> Result<CheckCoverage, &'static str> {
    let (mut findings, mut gaps) = (0_u64, 0_u64);
    let mut evidence = Vec::new();
    let mut reason = "directory-inspected";
    for result in results {
        let value = match result {
            Ok(value) => value,
            Err(error) => {
                gaps = gaps.checked_add(1).ok_or("collector-count-overflow")?;
                reason = error;
                evidence.push(json!({"error":error}));
                continue;
            }
        };
        let open = super::checks::count(&value, "open_findings")?;
        let missing = super::checks::length(&value, "uninspected")?;
        findings = findings
            .checked_add(open)
            .ok_or("collector-count-overflow")?;
        gaps = gaps
            .checked_add(missing)
            .ok_or("collector-count-overflow")?;
        // Retain only receipt hashes, never up to 256 full private receipts.
        let digest = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&value).map_err(|_| "collector-invalid-output")?)
        );
        evidence.push(json!({"receipt_sha256":digest}));
    }
    let status = if gaps > 0 {
        "incomplete"
    } else if findings > 0 {
        "findings"
    } else {
        "completed"
    };
    Ok(check(
        "content-disclosure",
        "selected-mounted-roots",
        status,
        reason,
        findings,
        gaps,
        Some(&json!(evidence)),
    ))
}
