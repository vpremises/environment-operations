//! Transport-neutral execution contracts; no process or network operations.

mod control;
mod helpers;
mod lease;
mod result;
pub use control::*;
pub use lease::*;
pub use result::*;
mod control_validation;
mod lease_request_validation;
mod lease_validation;
mod result_validation;

/// Semantic validation for a closed execution contract.
pub trait ValidateExecution {
    /// Check execution constraints without performing the operation.
    ///
    /// # Errors
    /// Returns an error when identity, state or evidence fields are invalid.
    fn validate_execution(&self) -> Result<(), ExecutionContractError>;
}
