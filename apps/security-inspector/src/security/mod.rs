//! One-shot local reporting preserves missing security evidence explicitly.

mod model;
pub use model::{CheckCoverage, SecurityReport};

use crate::{observe, validation::valid_identifier, ObserverConfig};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Collect local metadata and produce a portable security coverage report.
///
/// # Errors
/// Rejects invalid environment identifiers, unrepresentable timestamps and
/// configuration serialization errors. No private values appear in errors.
pub fn security_report(
    config: &ObserverConfig,
    environment_id: &str,
) -> Result<SecurityReport, &'static str> {
    if !valid_identifier(environment_id) {
        return Err("environment identifier is invalid");
    }
    let started_at_unix_ms = timestamp()?;
    let bytes = serde_json::to_vec(config).map_err(|_| "configuration cannot be serialized")?;
    let configuration_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let observation = observe(config);
    let mut checks = vec![CheckCoverage {
        id: "filesystem-metadata",
        scope: "selected-mounted-roots",
        finding_count: 0,
        gap_count: u64::from(!observation.ok),
        evidence_sha256: None,
        status: if observation.ok {
            "completed"
        } else {
            "incomplete"
        },
        reason: if observation.ok {
            "collected"
        } else {
            "observation-incomplete"
        },
    }];
    for id in [
        "content-disclosure",
        "network-exposure",
        "isolation-boundary",
    ] {
        checks.push(CheckCoverage {
            id,
            scope: "not-collected",
            finding_count: 0,
            gap_count: 1,
            evidence_sha256: None,
            status: "unsupported",
            reason: "collector-not-available",
        });
    }
    Ok(SecurityReport {
        schema: "vpremises-security/report/v2",
        tool_version: env!("CARGO_PKG_VERSION"),
        environment_id: environment_id.to_owned(),
        operating_system: std::env::consts::OS,
        started_at_unix_ms,
        finished_at_unix_ms: timestamp()?,
        configuration_sha256,
        outcome: "incomplete",
        external_actions: false,
        checks,
        observation,
    })
}

pub(crate) fn timestamp() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock precedes the Unix epoch")?
        .as_millis()
        .try_into()
        .map_err(|_| "system timestamp exceeds the supported range")
}
