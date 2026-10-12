//! Validate identities, state and evidence without executing actions.

use super::{
    helpers::{digest, identifier, require, schema_id},
    ExecutionContractError, ExecutionLeaseRequestV1, ValidateExecution,
};
use crate::EXECUTION_LEASE_REQUEST_SCHEMA_V1;

impl ValidateExecution for ExecutionLeaseRequestV1 {
    fn validate_execution(&self) -> Result<(), ExecutionContractError> {
        require(self.schema == EXECUTION_LEASE_REQUEST_SCHEMA_V1, "schema")?;
        for value in [
            &self.request_id,
            &self.invocation_id,
            &self.workspace_ref,
            &self.idempotency_key,
            &self.placement.provider_ref,
            &self.placement.environment_ref,
            &self.placement.resource_profile_ref,
        ] {
            require(identifier(value), "identifier")?;
        }
        require(schema_id(&self.action_id), "action_id")?;
        require(digest(&self.action_digest_sha256), "action_digest")?;
        require(digest(&self.effective_grant_digest_sha256), "grant_digest")
    }
}
