//! Evaluate a pinned, fresh supplied observation without claiming live host discovery.
use super::{
    checks::check,
    config::BoundaryCollector,
    io::{self, Scratch},
    runner,
};
use crate::CheckCoverage;
use sha2::{Digest, Sha256};
use std::{ffi::OsStr, io::Write};

pub(super) fn collect(
    config: &BoundaryCollector,
    now: u64,
    budget: &super::budget::Budget,
) -> Result<CheckCoverage, &'static str> {
    let tool = budget.tool(&config.tool)?;
    if !(1..=86400).contains(&config.max_age_seconds)
        || config.observed_at_unix_ms > now
        || now - config.observed_at_unix_ms > config.max_age_seconds * 1000
    {
        return Err("boundary-observation-stale");
    }
    let bytes = io::read(&config.input, 131_072)?;
    if format!("{:x}", Sha256::digest(&bytes)) != config.input_sha256 {
        return Err("boundary-input-pin-mismatch");
    }
    let document: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "boundary-input-invalid")?;
    let declared = document["generated_at"]
        .as_str()
        .ok_or("boundary-input-invalid")?;
    if super::time::unix_ms(declared)? != config.observed_at_unix_ms {
        return Err("boundary-observation-time-mismatch");
    }
    let scratch = Scratch::create()?;
    let input = scratch.0.join("boundary.json");
    io::output(&input)?
        .write_all(&bytes)
        .map_err(|_| "input-copy-failed")?;
    let (code, value) = runner::run(
        &tool,
        &[OsStr::new("evaluate"), input.as_os_str()],
        None,
        &scratch,
    )?;
    if code != 0 {
        return Err("collector-invalid-output");
    }
    let finished = crate::security::timestamp()?;
    if config.observed_at_unix_ms > finished
        || finished - config.observed_at_unix_ms > config.max_age_seconds * 1000
    {
        return Err("boundary-observation-stale");
    }
    let (findings, gaps) = super::boundary_receipt::validate(&value, &document)?;
    let status = if gaps > 0 {
        "incomplete"
    } else if findings > 0 {
        "findings"
    } else {
        "completed"
    };
    Ok(check(
        "isolation-boundary",
        "operator-supplied-boundary-observation",
        status,
        "supplied-boundary-evaluated",
        findings,
        gaps,
        Some(&value),
    ))
}
