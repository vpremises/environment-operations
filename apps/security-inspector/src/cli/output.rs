//! CLI failures remain structured and avoid leaking provider paths through stderr.

use serde::Serialize;
use serde_json::{json, Value};

pub(super) fn emit<T: Serialize>(value: &T, jsonl: bool) {
    println!(
        "{}",
        if jsonl {
            serde_json::to_string(value)
        } else {
            serde_json::to_string_pretty(value)
        }
        .expect("command output serializes")
    );
}

pub(super) fn report_request_error(code: &str, message: &str) -> Value {
    json!({
        "schema": "vpremises://errors/mounted-sharepoint-report/v1",
        "ok": false,
        "code": code,
        "message": message,
        "external_actions": false
    })
}

pub(super) fn usage_error() -> Value {
    json!({
        "schema_version": "vpremises.error/v1",
        "ok": false,
        "code": "vpremises.cli.usage",
        "message": "usage: vpremises-security doctor | init <new-private-directory> <absolute-root> | audit <audit.json> <environment-id> [--jsonl] | observe <config.json> | report <config.json> <environment-id> [--jsonl] | acquire-mounted-sharepoint-report <request.json> | oss repository|exclusions <root> [--policy <file>] | oss package npm|cargo <root> [--archive <file>] [--audit <file>]"
    })
}
