//! Terminal execution outcomes and content-free output references.

use serde::{Deserialize, Serialize};

/// Execution Outcome version 1 contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionOutcomeV1 {
    /// The operation completed with the required evidence.
    Completed,
    /// The operation failed.
    Failed,
    /// Policy rejected the operation.
    Denied,
    /// The operation was explicitly cancelled.
    Cancelled,
}

/// Projection Reference version 1 contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionReferenceV1 {
    /// Opaque identity of the service that owns the projection.
    pub owner_id: String,
    /// Opaque reference to a service-owned projection.
    pub projection_ref: String,
    /// Versioned schema identifier of the referenced projection.
    pub schema_id: String,
    /// Lowercase SHA-256 digest of the referenced projection bytes.
    pub digest_sha256: String,
}

/// Execution Result version 1 contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionResultV1 {
    /// Versioned contract URI.
    pub schema: String,
    /// Opaque identity of the accepted execution lease.
    pub lease_id: String,
    /// Opaque identity shared by the requested execution and its evidence.
    pub invocation_id: String,
    /// Lowercase SHA-256 digest binding the exact action definition.
    pub action_digest_sha256: String,
    /// Revision of the execution state represented by this evidence.
    pub state_revision: u64,
    /// Terminal operation outcome; distinct from report availability.
    pub outcome: ExecutionOutcomeV1,
    /// Content-free reference required only for a completed operation.
    pub output: Option<ProjectionReferenceV1>,
    /// Versioned reason identifier required for non-completed outcomes.
    pub reason_id: Option<String>,
}
