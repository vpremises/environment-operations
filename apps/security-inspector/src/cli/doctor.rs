//! Describe executable capabilities without claiming endpoint security.
use serde_json::{json, Value};

pub(super) fn doctor() -> Value {
    json!({
        "schema_version": "vpremises.doctor/v1",
        "healthy": true,
        "health_scope": "executable-contract-only",
        "configured_audit": {
            "requires_external_collectors": true,
            "content_inspection": true,
            "network_scope": "current-linux-network-namespace",
            "boundary_scope": "operator-supplied-boundary-observation",
            "security_verdict": "not-run"
        },
        "capabilities": {
            "metadata_only": true,
            "bounded_traversal": true,
            "symlink_following": false,
            "path_disclosure": false,
            "content_reading": false,
            "network_access": false
        }
    })
}
