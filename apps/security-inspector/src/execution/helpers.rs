//! Lexical checks for execution identities and hashes.

use super::ExecutionContractError;

pub(super) fn require(value: bool, field: &'static str) -> Result<(), ExecutionContractError> {
    value.then_some(()).ok_or(ExecutionContractError(field))
}

pub(super) fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

pub(super) fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn schema_id(value: &str) -> bool {
    value.len() <= 256
        && value.starts_with("hathq://")
        && value.rsplit_once("/v").is_some_and(|(_, version)| {
            !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
        })
}
