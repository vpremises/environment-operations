//! Aggregate metadata results exclude file names and contents.

use super::Diagnostic;
use serde::Serialize;

/// Aggregate-only result for one observation attempt.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObservationReport {
    /// Schema version.
    pub schema_version: &'static str,
    /// Whether metadata collection completed; not a security certification.
    pub ok: bool,
    /// Policy.
    pub policy: ReportedPolicy,
    /// Totals.
    pub totals: ObservationTotals,
    /// Roots.
    pub roots: Vec<RootObservation>,
    /// Diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Effective disclosure policy recorded in every result.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReportedPolicy {
    /// Access mode.
    pub access_mode: &'static str,
    /// Disclosure.
    pub disclosure: &'static str,
    /// Symlink policy.
    pub symlink_policy: &'static str,
}

/// Aggregate metadata counters that disclose no entry names.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct ObservationTotals {
    /// Entries examined.
    pub entries_examined: u64,
    /// Directories.
    pub directories: u64,
    /// Files.
    pub files: u64,
    /// Symlinks skipped.
    pub symlinks_skipped: u64,
    /// Other entries.
    pub other_entries: u64,
    /// Total file bytes.
    pub total_file_bytes: u64,
    /// Max depth reached.
    pub max_depth_reached: u32,
}

/// Completion state and counters for one opaque root ID.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RootObservation {
    /// Opaque identifier of an explicitly selected root.
    pub root_id: String,
    /// Whether this root was observed within every configured limit.
    pub complete: bool,
    /// Totals.
    pub totals: ObservationTotals,
}
