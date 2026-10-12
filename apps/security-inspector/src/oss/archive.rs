//! Inspect gzip-tar structures without extraction, under member and decompression budgets.
use super::{archive_entries, document, OssReport};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

pub(super) fn check(
    root: &Path,
    path: &Path,
    kind: &str,
    name: &str,
    version: &str,
    report: &mut OssReport,
) -> Result<(), &'static str> {
    let bytes = document::read(path, 64 * 1024 * 1024)?;
    report
        .evidence_sha256
        .push(format!("{:x}", Sha256::digest(&bytes)));
    let expanded = super::archive_stream::expand(&bytes)?;
    let mut tar = tar::Archive::new(expanded.as_slice());
    let entries = tar.entries().map_err(|_| "archive-invalid")?;
    let prefix = if kind == "npm" {
        "package".into()
    } else {
        format!("{name}-{version}")
    };
    let mut files = BTreeSet::new();
    let mut documents = BTreeMap::new();
    let mut size = 0_u64;
    for (index, entry) in entries.enumerate() {
        let mut entry = entry.map_err(|_| "archive-invalid-or-budget")?;
        size = size.checked_add(entry.size()).ok_or("archive-budget")?;
        if index >= 15_000 || size > 100_000_000 || entry.size() > 16_777_216 {
            return Err("archive-budget");
        }
        let path = entry.path().map_err(|_| "archive-path-invalid")?;
        let Some(path) = path.to_str() else {
            report.gap("archive-path-encoding", "");
            continue;
        };
        let path = path.trim_end_matches('/');
        let entry_type = entry.header().entry_type();
        if !super::paths::relative(path)
            || !path.starts_with(&(prefix.clone() + "/"))
            || !(entry_type.is_file() || entry_type.is_dir())
        {
            if path == prefix && entry_type.is_dir() {
                continue;
            }
            report.finding("unsafe-archive-member", "");
            continue;
        }
        let relative = path[prefix.len() + 1..].to_owned();
        if !entry_type.is_file() {
            continue;
        }
        if !files.insert(relative.clone()) {
            report.finding("duplicate-archive-file", &relative);
        }
        if super::paths::archive_forbidden(&relative) {
            report.finding("forbidden-archive-path", &relative);
        }
        if ["package.json", "Cargo.toml"].contains(&relative.as_str()) {
            if entry.size() > 1_048_576 {
                return Err("manifest-budget");
            }
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|_| "archive-invalid")?;
            documents.insert(relative, data);
        }
    }
    archive_entries::check(root, kind, name, version, &files, &documents, report)
}
