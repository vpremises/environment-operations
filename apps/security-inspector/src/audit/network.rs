//! Crowsi sees this Linux namespace, never the mounted Windows host's network.
use super::{
    checks::{check, length},
    config::NetworkCollector,
    io::{self, Scratch},
    runner,
};
use crate::CheckCoverage;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{ffi::OsStr, io::Write};

pub(super) fn collect(
    config: &NetworkCollector,
    budget: &super::budget::Budget,
) -> Result<CheckCoverage, &'static str> {
    let tool = budget.tool(&config.tool)?;
    let scratch = Scratch::create()?;
    // Pass immutable bounded bytes, not a caller-owned file/device descriptor.
    let baseline = config
        .baseline
        .as_ref()
        .map(|path| {
            let bytes = io::read(path, 1_048_576)?;
            let digest = format!("{:x}", Sha256::digest(&bytes));
            let copy = scratch.0.join("baseline.json");
            io::output(&copy)?
                .write_all(&bytes)
                .map_err(|_| "input-copy-failed")?;
            Ok::<_, &'static str>((copy, digest))
        })
        .transpose()?;
    let command = if config.baseline.is_some() {
        "evaluate"
    } else {
        "observe"
    };
    let (code, value) = runner::run(
        &tool,
        &[OsStr::new(command)],
        baseline.as_ref().map(|(path, _)| path.as_path()),
        &scratch,
    )?;
    if code != 0
        || value["schema"] != "crowsi://network/host-network-snapshot/v1"
        || value["external_actions"] != false
        || value["signal_trust"] != "unsigned-local"
    {
        return Err("collector-invalid-output");
    }
    let findings = length(&value, "findings")?;
    let status = match value["status"].as_str() {
        Some("unknown") => "incomplete",
        Some("observed") if config.baseline.is_some() && findings == 0 => "completed",
        Some("observed") if config.baseline.is_none() => "incomplete",
        Some("drifted") if config.baseline.is_some() && findings > 0 => "findings",
        _ => return Err("collector-invalid-output"),
    };
    let reason = if config.baseline.is_none() {
        "baseline-not-configured"
    } else {
        "baseline-evaluated"
    };
    Ok(check(
        "network-exposure",
        "current-linux-network-namespace",
        status,
        reason,
        findings,
        u64::from(status == "incomplete"),
        Some(&json!({"snapshot":value,"baseline_sha256":baseline.map(|(_,digest)|digest)})),
    ))
}
