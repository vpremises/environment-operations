//! Execution contracts bind operations to identity, revision and immutable evidence.

use vpremises::{
    ExecutionControlKindV1, ExecutionControlRequestV1, ExecutionLeaseRequestV1, ExecutionLeaseV1,
    ExecutionOutcomeV1, ExecutionPlacementV1, ExecutionResultV1, ExecutionStateV1,
    ProjectionReferenceV1, ValidateExecution, EXECUTION_CONTROL_SCHEMA_V1,
    EXECUTION_LEASE_REQUEST_SCHEMA_V1, EXECUTION_RESULT_SCHEMA_V1,
};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn lease_request() -> ExecutionLeaseRequestV1 {
    ExecutionLeaseRequestV1 {
        schema: EXECUTION_LEASE_REQUEST_SCHEMA_V1.to_owned(),
        request_id: "request-1".to_owned(),
        invocation_id: "invoke-1".to_owned(),
        workspace_ref: "workspace-1".to_owned(),
        action_id: "hathq://vocabulary/action/review-source/v1".to_owned(),
        action_digest_sha256: DIGEST.to_owned(),
        effective_grant_digest_sha256: DIGEST.to_owned(),
        placement: ExecutionPlacementV1 {
            provider_ref: "vpremises-provider-1".to_owned(),
            environment_ref: "environment-1".to_owned(),
            resource_profile_ref: "profile-small".to_owned(),
        },
        expected_state_revision: 0,
        idempotency_key: "lease-1".to_owned(),
    }
}

#[test]
fn execution_lease_is_action_bound_and_has_no_native_command_escape_hatch() {
    let request = lease_request();
    assert!(request.validate_execution().is_ok());
    let mut value = serde_json::to_value(request).expect("serialize");
    value
        .as_object_mut()
        .expect("object")
        .insert("command".to_owned(), serde_json::json!(["sh", "-c", "id"]));
    assert!(serde_json::from_value::<ExecutionLeaseRequestV1>(value).is_err());
}

#[test]
fn result_and_cancel_recovery_are_revision_and_idempotency_bound() {
    let lease = ExecutionLeaseV1 {
        schema: vpremises::EXECUTION_LEASE_SCHEMA_V1.to_owned(),
        lease_id: "lease-1".to_owned(),
        request_id: "request-1".to_owned(),
        request_digest_sha256: DIGEST.to_owned(),
        invocation_id: "invoke-1".to_owned(),
        action_digest_sha256: DIGEST.to_owned(),
        state_revision: 1,
        state: ExecutionStateV1::Prepared,
        placement: lease_request().placement,
    };
    assert!(lease.validate_execution().is_ok());

    let result = ExecutionResultV1 {
        schema: EXECUTION_RESULT_SCHEMA_V1.to_owned(),
        lease_id: "lease-1".to_owned(),
        invocation_id: "invoke-1".to_owned(),
        action_digest_sha256: DIGEST.to_owned(),
        state_revision: 2,
        outcome: ExecutionOutcomeV1::Completed,
        output: Some(ProjectionReferenceV1 {
            owner_id: "hat-source-curator".to_owned(),
            projection_ref: "projection-1".to_owned(),
            schema_id: "hathq://hat-source-curator/review-source-output/v1".to_owned(),
            digest_sha256: DIGEST.to_owned(),
        }),
        reason_id: None,
    };
    assert!(result.validate_execution().is_ok());

    for kind in [
        ExecutionControlKindV1::ReadStatus,
        ExecutionControlKindV1::Cancel,
        ExecutionControlKindV1::Recover,
        ExecutionControlKindV1::Cleanup,
    ] {
        let control = ExecutionControlRequestV1 {
            schema: EXECUTION_CONTROL_SCHEMA_V1.to_owned(),
            request_id: "control-1".to_owned(),
            lease_id: "lease-1".to_owned(),
            invocation_id: "invoke-1".to_owned(),
            expected_state_revision: 2,
            idempotency_key: "control-1".to_owned(),
            kind,
        };
        assert!(control.validate_execution().is_ok());
    }
}
