//! Validate identities, state and evidence without executing actions.

use super::{
    helpers::{digest, identifier, require},
    ExecutionContractError, ExecutionLeaseV1, ExecutionStateV1, ValidateExecution,
};
use crate::EXECUTION_LEASE_SCHEMA_V1;

impl ValidateExecution for ExecutionLeaseV1 {
    fn validate_execution(&self) -> Result<(), ExecutionContractError> {
        require(self.schema == EXECUTION_LEASE_SCHEMA_V1, "schema")?;
        for value in [&self.lease_id, &self.request_id, &self.invocation_id] {
            require(identifier(value), "lease identity")?;
        }
        require(
            digest(&self.request_digest_sha256) && digest(&self.action_digest_sha256),
            "lease digest",
        )?;
        require(
            matches!(self.state, ExecutionStateV1::Prepared),
            "lease state",
        )?;
        for value in [
            &self.placement.provider_ref,
            &self.placement.environment_ref,
            &self.placement.resource_profile_ref,
        ] {
            require(identifier(value), "placement")?;
        }
        Ok(())
    }
}
