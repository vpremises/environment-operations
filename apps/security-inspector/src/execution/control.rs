//! Explicit execution control requests and bounded errors.

use serde::{Deserialize, Serialize};

/// Execution Control Kind version 1 contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionControlKindV1 {
    /// Read the current operation state without mutation.
    ReadStatus,
    /// Request cancellation of the selected operation.
    Cancel,
    /// Request explicit recovery of an unresolved operation.
    Recover,
    /// Request explicit cleanup of operation resources.
    Cleanup,
}
/// Execution Control Request version 1 contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionControlRequestV1 {
    /// Versioned contract URI.
    pub schema: String,
    /// Opaque identity of this request.
    pub request_id: String,
    /// Opaque identity of the accepted execution lease.
    pub lease_id: String,
    /// Opaque identity shared by the requested execution and its evidence.
    pub invocation_id: String,
    /// Revision the caller observed before requesting this operation.
    pub expected_state_revision: u64,
    /// Opaque key preventing a retry from creating a second operation.
    pub idempotency_key: String,
    /// Explicit control operation requested by the caller.
    pub kind: ExecutionControlKindV1,
}

/// Execution Contract Error contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContractError(
    /// Stable contract field or invariant that failed validation.
    pub &'static str,
);
