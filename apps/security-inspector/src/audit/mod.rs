//! WSL/Linux audit orchestrates local engines and preserves incomplete coverage.
mod boundary;
mod boundary_receipt;
mod budget;
mod checks;
mod config;
mod content;
mod content_receipt;
pub(crate) mod io;
mod load;
pub use load::{load_audit_config, read_local_document};
mod network;
pub(crate) mod oss_content;
mod process;
mod runner;
mod setup;
pub use setup::initialize_bundle;
mod time;
use crate::{security_report, CheckCoverage, SecurityReport};
pub use config::{
    AuditConfig, BoundaryCollector, Collectors, ContentCollector, NetworkCollector, Tool,
};
use sha2::{Digest, Sha256};

/// Audit explicitly mounted roots with configured, pinned local collectors.
/// # Errors
/// Reject invalid configuration contracts and environment identifiers.
pub fn audit(config: &AuditConfig, environment: &str) -> Result<SecurityReport, &'static str> {
    if config.schema != "vpremises-security/audit/v1" {
        return Err("audit-schema-invalid");
    }
    if !(1..=3600).contains(&config.collector_budget_seconds) {
        return Err("collector-budget-invalid");
    }
    let mut report = security_report(&config.observer, environment)?;
    let budget = budget::Budget::new(config.collector_budget_seconds)?;
    report.schema = "vpremises-security/report/v2";
    report.configuration_sha256 = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(config).map_err(|_| "audit-config-invalid")?)
    );
    let mut checks = vec![report.checks.remove(0)];
    let content = config.collectors.content.as_ref().map_or_else(
        || {
            Ok(checks::missing(
                "content-disclosure",
                "selected-mounted-roots",
            ))
        },
        |collector| {
            if report.observation.ok {
                content::collect(collector, &config.observer, &budget)
            } else {
                Err("observation-incomplete")
            }
        },
    );
    checks.push(result(
        "content-disclosure",
        "selected-mounted-roots",
        content,
    ));
    let network = config.collectors.network.as_ref().map_or_else(
        || {
            Ok(checks::missing(
                "network-exposure",
                "current-linux-network-namespace",
            ))
        },
        |collector| network::collect(collector, &budget),
    );
    checks.push(result(
        "network-exposure",
        "current-linux-network-namespace",
        network,
    ));
    let boundary = config.collectors.boundary.as_ref().map_or_else(
        || {
            Ok(checks::missing(
                "isolation-boundary",
                "operator-supplied-boundary-observation",
            ))
        },
        |collector| boundary::collect(collector, crate::security::timestamp()?, &budget),
    );
    checks.push(result(
        "isolation-boundary",
        "operator-supplied-boundary-observation",
        boundary,
    ));
    report.outcome = if checks
        .iter()
        .any(|c| matches!(c.status, "incomplete" | "unsupported"))
    {
        "incomplete"
    } else if checks.iter().any(|c| c.finding_count > 0) {
        "findings"
    } else {
        "passed"
    };
    report.checks = checks;
    report.finished_at_unix_ms = crate::security::timestamp()?;
    Ok(report)
}
fn result(
    id: &'static str,
    scope: &'static str,
    value: Result<CheckCoverage, &'static str>,
) -> CheckCoverage {
    value.unwrap_or_else(|reason| checks::check(id, scope, "incomplete", reason, 0, 1, None))
}

#[cfg(test)]
mod tests;
