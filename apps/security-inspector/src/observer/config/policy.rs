//! Global policy is closed to metadata-only, no-follow, and bounded traversal.

use crate::{
    model::{Diagnostic, ObserverConfig},
    observer::diagnostic,
    CONFIG_SCHEMA_VERSION,
};

pub(super) fn validate(config: &ObserverConfig) -> Vec<Diagnostic> {
    let mut findings = Vec::new();
    if config.schema_version != CONFIG_SCHEMA_VERSION {
        findings.push(diagnostic::create(
            "vpremises.schema.unsupported",
            None,
            "$.schema_version",
            format!(
                "expected schema version {CONFIG_SCHEMA_VERSION}, got {}",
                config.schema_version
            ),
        ));
    }
    require(
        config.policy.metadata_only,
        "vpremises.policy.metadata-only-required",
        "$.policy.metadata_only",
        "metadata_only must be true",
        &mut findings,
    );
    require(
        !config.policy.follow_symlinks,
        "vpremises.policy.symlink-follow-forbidden",
        "$.policy.follow_symlinks",
        "follow_symlinks must be false",
        &mut findings,
    );
    require(
        !config.roots.is_empty(),
        "vpremises.roots.empty",
        "$.roots",
        "at least one explicit allowed root is required",
        &mut findings,
    );
    require(
        config.roots.len() <= 256,
        "vpremises.roots.too-many",
        "$.roots",
        "at most 256 explicit allowed roots are permitted",
        &mut findings,
    );
    require(
        config.limits.max_depth <= 64,
        "vpremises.limit.depth-out-of-range",
        "$.limits.max_depth",
        "max_depth must be between 0 and 64",
        &mut findings,
    );
    require(
        (1..=1_000_000).contains(&config.limits.max_entries),
        "vpremises.limit.entries-out-of-range",
        "$.limits.max_entries",
        "max_entries must be between 1 and 1000000",
        &mut findings,
    );
    require(
        (1..=1_000_000_000_000_000).contains(&config.limits.max_total_bytes),
        "vpremises.limit.bytes-out-of-range",
        "$.limits.max_total_bytes",
        "max_total_bytes must be between 1 and 1000000000000000",
        &mut findings,
    );
    findings
}

fn require(
    condition: bool,
    code: &str,
    field: &str,
    message: &str,
    findings: &mut Vec<Diagnostic>,
) {
    if !condition {
        findings.push(diagnostic::create(code, None, field, message));
    }
}
