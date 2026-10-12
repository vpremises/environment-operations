//! Observation contracts disclose aggregate metadata and never filesystem names or paths.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Closed policy, root allowlist, and global traversal limits.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ObserverConfig {
    /// Schema version.
    pub schema_version: String,
    /// Roots.
    pub roots: Vec<AllowedRoot>,
    /// Limits.
    pub limits: ObservationLimits,
    /// Policy.
    pub policy: ObservationPolicy,
}

/// One explicitly allowed observation root.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AllowedRoot {
    /// Id.
    pub id: String,
    /// Explicitly allowed local path; excluded from observation reports.
    pub path: PathBuf,
}

/// Global traversal ceilings shared by every configured root.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ObservationLimits {
    /// Maximum traversal depth below each allowed root.
    pub max_depth: u32,
    /// Maximum entries examined across all configured roots.
    pub max_entries: u64,
    /// Maximum aggregate size of observed regular files.
    pub max_total_bytes: u64,
}

/// Mandatory metadata-only and no-follow policy.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ObservationPolicy {
    /// Must remain true; observation does not read file contents.
    pub metadata_only: bool,
    /// Must remain false; traversal skips linked entries.
    pub follow_symlinks: bool,
}

/// Machine-readable observer validation or traversal finding.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Diagnostic {
    /// Code.
    pub code: String,
    /// Opaque identifier of an explicitly selected root.
    pub root_id: Option<String>,
    /// Field.
    pub field: String,
    /// Message.
    pub message: String,
}
