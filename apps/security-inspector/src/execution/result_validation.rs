//! Validate identities, state and evidence without executing actions.

use super::{
    helpers::{digest, identifier, require, schema_id},
    ExecutionContractError, ExecutionOutcomeV1, ExecutionResultV1, ValidateExecution,
};
use crate::EXECUTION_RESULT_SCHEMA_V1;

impl ValidateExecution for ExecutionResultV1 {
    fn validate_execution(&self) -> Result<(), ExecutionContractError> {
        require(self.schema == EXECUTION_RESULT_SCHEMA_V1, "schema")?;
        require(
            identifier(&self.lease_id) && identifier(&self.invocation_id),
            "identity",
        )?;
        require(digest(&self.action_digest_sha256), "action_digest")?;
        match self.outcome {
            ExecutionOutcomeV1::Completed => {
                require(
                    self.output.is_some() && self.reason_id.is_none(),
                    "completed",
                )?;
            }
            _ => require(
                self.output.is_none() && self.reason_id.as_ref().is_some_and(|id| schema_id(id)),
                "terminal",
            )?,
        }
        if let Some(output) = &self.output {
            require(identifier(&output.owner_id), "output owner")?;
            require(identifier(&output.projection_ref), "projection_ref")?;
            require(
                schema_id(&output.schema_id) && digest(&output.digest_sha256),
                "output",
            )?;
        }
        Ok(())
    }
}
