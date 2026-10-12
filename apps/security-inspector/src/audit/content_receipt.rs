//! A fresh directory scan cannot hide findings behind reviewed counts or gaps.
use super::checks::{count, length};
use serde_json::Value;

pub(super) fn validate(code: i32, value: &Value) -> Result<(u64, u64), &'static str> {
    if value["schema"] != "zixcel://repository-security/receipt/v1"
        || value["scope"] != "directory-secret-disclosure/v1"
        || value["engines"]["gitleaks"]["status"] != "completed"
        || count(value, "reviewed_findings")? != 0
    {
        return Err("collector-invalid-output");
    }
    let open = count(value, "open_findings")?;
    let missing = length(value, "uninspected")?;
    if open != length(value, "findings")? {
        return Err("collector-invalid-output");
    }
    let expected = if missing > 0 {
        "incomplete"
    } else if open > 0 {
        "findings"
    } else {
        "passed"
    };
    if value["status"] != expected
        || ![0, 1].contains(&code)
        || (code == 0) != (expected == "passed")
    {
        return Err("collector-invalid-output");
    }
    Ok((open, missing))
}
