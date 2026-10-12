//! Immutable placement and lease contracts.

use serde::{Deserialize, Serialize};

/// Execution Placement version 1 contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlacementV1 {
    /// Registered execution provider identifier.
    pub provider_ref: String,
    /// Registered execution environment identifier.
    pub environment_ref: String,
    /// Registered resource-limit profile identifier.
    pub resource_profile_ref: String,
}

/// Execution Lease Request version 1 contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionLeaseRequestV1 {
    /// Versioned contract URI.
    pub schema: String,
    /// Opaque identity of this request.
    pub request_id: String,
    /// Opaque identity shared by the requested execution and its evidence.
    pub invocation_id: String,
    /// Registered workspace identifier; not a filesystem path.
    pub workspace_ref: String,
    /// Versioned identifier of the permitted action.
    pub action_id: String,
    /// Lowercase SHA-256 digest binding the exact action definition.
    pub action_digest_sha256: String,
    /// Lowercase SHA-256 digest of the effective authorization grant.
    pub effective_grant_digest_sha256: String,
    /// Explicit provider, environment and resource-profile selection.
    pub placement: ExecutionPlacementV1,
    /// Revision the caller observed before requesting this operation.
    pub expected_state_revision: u64,
    /// Opaque key preventing a retry from creating a second operation.
    pub idempotency_key: String,
}

/// Execution State version 1 contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionStateV1 {
    /// The operation is authorized but has not started.
    Prepared,
    /// The operation is executing.
    Running,
    /// Current execution evidence is unavailable.
    Unknown,
    /// The operation completed with the required evidence.
    Completed,
    /// The operation failed.
    Failed,
    /// The operation was explicitly cancelled.
    Cancelled,
}

/// Execution Lease version 1 contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionLeaseV1 {
    /// Versioned contract URI.
    pub schema: String,
    /// Opaque identity of the accepted execution lease.
    pub lease_id: String,
    /// Opaque identity of this request.
    pub request_id: String,
    /// Lowercase SHA-256 digest binding the accepted request.
    pub request_digest_sha256: String,
    /// Opaque identity shared by the requested execution and its evidence.
    pub invocation_id: String,
    /// Lowercase SHA-256 digest binding the exact action definition.
    pub action_digest_sha256: String,
    /// Revision of the execution state represented by this evidence.
    pub state_revision: u64,
    /// Lifecycle state of the execution lease.
    pub state: ExecutionStateV1,
    /// Explicit provider, environment and resource-profile selection.
    pub placement: ExecutionPlacementV1,
}
