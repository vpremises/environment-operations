//! Collector summaries expose counts and evidence hashes, never raw findings.
use crate::CheckCoverage;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(super) fn missing(id: &'static str, scope: &'static str) -> CheckCoverage {
    check(
        id,
        scope,
        "incomplete",
        "collector-not-configured",
        0,
        1,
        None,
    )
}
pub(super) fn check(
    id: &'static str,
    scope: &'static str,
    status: &'static str,
    reason: &'static str,
    findings: u64,
    gaps: u64,
    evidence: Option<&Value>,
) -> CheckCoverage {
    CheckCoverage {
        id,
        scope,
        status,
        reason,
        finding_count: findings,
        gap_count: gaps,
        evidence_sha256: evidence.map(|v| {
            format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(v).expect("collector evidence serializes"))
            )
        }),
    }
}
pub(super) fn length(value: &Value, key: &str) -> Result<u64, &'static str> {
    value[key]
        .as_array()
        .and_then(|v| u64::try_from(v.len()).ok())
        .ok_or("collector-invalid-output")
}
pub(super) fn count(value: &Value, key: &str) -> Result<u64, &'static str> {
    value[key].as_u64().ok_or("collector-invalid-output")
}
