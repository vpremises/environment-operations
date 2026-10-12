//! Release routes are checked independently from registration and legal ownership.
use super::OssReport;
use serde_json::Value;
use std::path::Path;

pub(super) fn check(root: &Path, kind: &str, value: &Value, report: &mut OssReport) {
    let package = if kind == "npm" {
        value
    } else {
        &value["package"]
    };
    let version = package["version"].as_str().unwrap_or_default();
    if package["name"].as_str().is_none_or(str::is_empty) || !version_valid(version) {
        report.finding("package-identity-invalid", "package");
    }
    if !["license", "license-file"]
        .iter()
        .any(|key| package[key].as_str().is_some_and(|s| !s.trim().is_empty()))
    {
        report.finding("package-license-missing", "package");
    }
    let repository = package["repository"]
        .as_str()
        .or_else(|| package["repository"]["url"].as_str())
        .unwrap_or_default();
    if !repository_valid(repository, kind) {
        report.finding("package-repository-invalid", "repository");
    }
    if kind == "npm" {
        super::npm_manifest::npm(value, report);
    } else {
        super::cargo_manifest::cargo(root, value, report);
    }
}
fn repository_valid(value: &str, kind: &str) -> bool {
    let value = if kind == "npm" {
        value.strip_prefix("git+").unwrap_or(value)
    } else {
        value
    };
    let Some(path) = value.strip_prefix("https://github.com/") else {
        return false;
    };
    let path = if kind == "npm" {
        let Some(p) = path.strip_suffix(".git") else {
            return false;
        };
        p
    } else {
        path
    };
    path.split('/').count() == 2
        && path.split('/').all(|p| {
            !p.is_empty()
                && ![".", ".."].contains(&p)
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
fn version_valid(value: &str) -> bool {
    let (base, suffix) = value
        .split_once('-')
        .map_or((value, None), |(a, b)| (a, Some(b)));
    base.split('.').count() == 3
        && base
            .split('.')
            .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        && suffix.is_none_or(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
        })
}
