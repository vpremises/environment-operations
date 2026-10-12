//! Explicit external collectors; private settings and inputs stay outside source.
use crate::ObserverConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A closed, operator-selected mounted-directory audit configuration.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditConfig {
    /// Configuration contract identifier.
    pub schema: String,
    /// Selected roots and global metadata traversal limits.
    pub observer: ObserverConfig,
    /// Explicit detectors; missing detectors remain incomplete.
    pub collectors: Collectors,
    /// Shared collector execution budget, 1-3600 seconds; defaults to 300.
    #[serde(default = "default_budget")]
    pub collector_budget_seconds: u64,
}
fn default_budget() -> u64 {
    300
}
/// Optional local collectors; never discover or download executables implicitly.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Collectors {
    /// Zixcel/Gitleaks directory content inspection.
    pub content: Option<ContentCollector>,
    /// Crowsi current-namespace network inspection.
    pub network: Option<NetworkCollector>,
    /// Crowsi evaluation of a fresh operator-supplied boundary observation.
    pub boundary: Option<BoundaryCollector>,
}
/// A pinned ELF executable with a bounded runtime.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    /// Absolute physical executable path.
    pub executable: PathBuf,
    /// Expected lowercase SHA-256 of executable bytes.
    pub sha256: String,
    /// Per-invocation timeout, from 1 to 600 seconds.
    pub timeout_seconds: u64,
}
/// All directory entries are sent only to the local pinned Zixcel adapter.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContentCollector {
    /// Zixcel executable identity.
    pub tool: Tool,
    /// External Zixcel configuration including the pinned Gitleaks engine.
    pub settings: PathBuf,
}
/// Network evaluation is scoped to this Linux process's namespace.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkCollector {
    /// Crowsi host-network sensor identity.
    pub tool: Tool,
    /// External approved baseline; absent baseline means observation only.
    pub baseline: Option<PathBuf>,
}
/// A supplied observation is evaluated, never inferred from mounted files.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryCollector {
    /// Crowsi boundary-monitor identity.
    pub tool: Tool,
    /// Private Crowsi boundary-input/v1 document.
    pub input: PathBuf,
    /// Expected input digest prevents changing the observation after approval.
    pub input_sha256: String,
    /// Operator-declared observation time in UTC Unix milliseconds.
    pub observed_at_unix_ms: u64,
    /// Maximum permitted input age, from 1 to 86400 seconds.
    pub max_age_seconds: u64,
}
