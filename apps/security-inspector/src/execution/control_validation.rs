//! Validate identities, state and evidence without executing actions.

use super::{
    helpers::{identifier, require},
    ExecutionContractError, ExecutionControlRequestV1, ValidateExecution,
};
use crate::EXECUTION_CONTROL_SCHEMA_V1;

impl ValidateExecution for ExecutionControlRequestV1 {
    fn validate_execution(&self) -> Result<(), ExecutionContractError> {
        require(self.schema == EXECUTION_CONTROL_SCHEMA_V1, "schema")?;
        for value in [
            &self.request_id,
            &self.lease_id,
            &self.invocation_id,
            &self.idempotency_key,
        ] {
            require(identifier(value), "control identity")?;
        }
        Ok(())
    }
}
