//! Inspect only supplied content roots; publication gates never invoke other collectors.
use super::{budget::Budget, config::AuditConfig, content};
use crate::{observe, CheckCoverage};

pub(crate) fn collect(config: &AuditConfig) -> Result<CheckCoverage, &'static str> {
    if config.schema != "vpremises-security/audit/v1" {
        return Err("audit-schema-invalid");
    }
    let collector = config
        .collectors
        .content
        .as_ref()
        .ok_or("archive-content-missing")?;
    if !observe(&config.observer).ok {
        return Err("archive-observation-incomplete");
    }
    let budget = Budget::new(config.collector_budget_seconds)?;
    content::collect(collector, &config.observer, &budget)
}
