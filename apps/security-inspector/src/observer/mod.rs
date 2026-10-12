//! Metadata observation validates every root before scanning any of them.

mod config;
pub(crate) mod diagnostic;
mod scan;

use crate::model::{
    ObservationReport, ObservationTotals, ObserverConfig, ReportedPolicy, RootObservation,
};

/// Validates all roots and returns aggregate metadata without names or paths.
#[must_use]
pub fn observe(config: &ObserverConfig) -> ObservationReport {
    let (mut roots, mut diagnostics) = config::validate(config);
    let mut report = empty_report();
    if !diagnostics.is_empty() {
        diagnostics.sort_by(diagnostic::order);
        report.diagnostics = diagnostics;
        return report;
    }
    roots.sort_by(|left, right| left.id.cmp(&right.id));
    for root in roots {
        let mut root_result = RootObservation {
            root_id: root.id.clone(),
            complete: true,
            totals: ObservationTotals::default(),
        };
        if let Err(diagnostic) = scan::root(
            &root,
            &config.limits,
            &mut report.totals,
            &mut root_result.totals,
        ) {
            root_result.complete = false;
            report.diagnostics.push(diagnostic);
        }
        report.roots.push(root_result);
        if !report.diagnostics.is_empty() {
            break;
        }
    }
    report.diagnostics.sort_by(diagnostic::order);
    report.ok = report.diagnostics.is_empty();
    report
}

fn empty_report() -> ObservationReport {
    ObservationReport {
        schema_version: "vpremises.observation/v1",
        ok: false,
        policy: ReportedPolicy {
            access_mode: "metadata-only",
            disclosure: "aggregate-counts-only",
            symlink_policy: "skip",
        },
        totals: ObservationTotals::default(),
        roots: Vec::new(),
        diagnostics: Vec::new(),
    }
}

pub(crate) struct ValidatedRoot {
    pub(crate) id: String,
    pub(crate) canonical_path: std::path::PathBuf,
}
