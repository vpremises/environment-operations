//! Compare catalog identifiers only against an explicit public allowlist.
use super::{document, OssReport};
use std::path::Path;
pub(super) fn check(root: &Path, report: &mut OssReport) {
    let policy = root.join("public-package-policy.json");
    if !policy.try_exists().unwrap_or(true) {
        return;
    }
    let result = (|| {
        let value = document::load(&policy, report)?;
        let expected = strings(&value["repository_ids"])?;
        for name in ["catalog-v2.json", "public/catalog/v2/index.json"] {
            let path = root.join(name);
            if !path.try_exists().unwrap_or(true) {
                continue;
            }
            let value = document::load(&path, report)?;
            let entries = value["entries"].as_array().ok_or("catalog-invalid")?;
            let mut actual = entries
                .iter()
                .map(|v| {
                    v["repository_id"]
                        .as_str()
                        .map(str::to_string)
                        .ok_or("catalog-invalid")
                })
                .collect::<Result<Vec<_>, _>>()?;
            actual.sort();
            if actual != expected {
                report.finding("catalog-outside-public-allowlist", name);
            }
        }
        Ok::<_, &'static str>(())
    })();
    if let Err(code) = result {
        report.gap(code, "public-package-policy.json");
    }
}
fn strings(value: &serde_json::Value) -> Result<Vec<String>, &'static str> {
    let mut values = value
        .as_array()
        .ok_or("catalog-policy-invalid")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or("catalog-policy-invalid")
        })
        .collect::<Result<Vec<_>, _>>()?;
    values.sort();
    Ok(values)
}
