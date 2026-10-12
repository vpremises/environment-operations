//! Cross-check environment-level evidence instead of trusting summary counters.
use super::checks::count;
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn validate(value: &Value, input: &Value) -> Result<(u64, u64), &'static str> {
    if value["schema"] != "crowsi://network/boundary-snapshot/v1"
        || value["external_actions"] != false
        || value["generated_at"] != input["generated_at"]
    {
        return Err("collector-invalid-output");
    }
    let environments = value["environments"]
        .as_array()
        .ok_or("collector-invalid-output")?;
    let expected = input["environments"]
        .as_array()
        .ok_or("boundary-input-invalid")?;
    if environments.is_empty() || environments.len() != expected.len() {
        return Err("collector-invalid-output");
    }
    let (mut healthy, mut attention, mut unknown, mut ingress) = (0_u64, 0_u64, 0_u64, 0_u64);
    let mut ids = BTreeSet::new();
    for (environment, source) in environments.iter().zip(expected) {
        let id = environment["id"]
            .as_str()
            .ok_or("collector-invalid-output")?;
        if source["id"] != id || !ids.insert(id) {
            return Err("collector-invalid-output");
        }
        match environment["status"].as_str() {
            Some("healthy") => healthy += 1,
            Some("attention") => attention += 1,
            Some("unknown") => unknown += 1,
            _ => return Err("collector-invalid-output"),
        }
        ingress += u64::from(
            environment["public_ingress"]
                .as_bool()
                .ok_or("collector-invalid-output")?,
        );
    }
    let summary = &value["summary"];
    if count(summary, "environment_count")? != environments.len() as u64
        || count(summary, "healthy_count")? != healthy
        || count(summary, "attention_count")? != attention
        || count(summary, "unknown_count")? != unknown
        || value["overall_status"]
            != if attention > 0 {
                "attention"
            } else if unknown > 0 {
                "unknown"
            } else {
                "healthy"
            }
    {
        return Err("collector-invalid-output");
    }
    // Ingress is a separate exposure signal even for an isolated environment.
    Ok((
        attention
            .checked_add(ingress)
            .ok_or("collector-count-overflow")?,
        unknown,
    ))
}
