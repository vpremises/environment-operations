//! Explicit source limits cover maintained untracked additions without following links.
use super::{document, policy::RustSource, OssReport};
use std::{fs, os::fd::AsRawFd, path::Path};

pub(super) fn check(root: &Path, policy: &RustSource, report: &mut OssReport) {
    let mut budget = 200_000_usize;
    for directory in &policy.directories {
        walk(root, &root.join(directory), policy, report, &mut budget, 0);
    }
}
fn walk(
    root: &Path,
    path: &Path,
    policy: &RustSource,
    report: &mut OssReport,
    budget: &mut usize,
    depth: usize,
) {
    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    if depth > 64 || *budget == 0 {
        report.gap("source-traversal-budget", &relative);
        return;
    }
    let Ok(directory) = crate::audit::io::open(path) else {
        report.gap("source-directory-boundary", &relative);
        return;
    };
    let anchored = format!("/proc/self/fd/{}", directory.as_raw_fd());
    let Ok(entries) = fs::read_dir(anchored) else {
        report.gap("source-directory-unavailable", &relative);
        return;
    };
    for entry in entries {
        if *budget == 0 {
            report.gap("source-traversal-budget", &relative);
            break;
        }
        *budget -= 1;
        let Ok(entry) = entry else {
            report.gap("source-entry-unavailable", &relative);
            continue;
        };
        let path = path.join(entry.file_name());
        let name = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        let Ok(kind) = entry.file_type() else {
            report.gap("source-entry-unavailable", &name);
            continue;
        };
        if kind.is_symlink() {
            report.gap("source-symlink", &name);
            continue;
        }
        if kind.is_dir() {
            if !["target", "dist", "node_modules", ".git"]
                .iter()
                .any(|s| entry.file_name() == *s)
            {
                walk(root, &path, policy, report, budget, depth + 1);
            }
        } else if path.extension().is_some_and(|s| s == "rs") {
            match document::read(&path, 1_048_576)
                .and_then(|bytes| String::from_utf8(bytes).map_err(|_| "source-invalid-encoding"))
            {
                Ok(text) => {
                    if text.lines().count() > policy.max_lines {
                        report.finding("source-line-limit", &name);
                    }
                    if policy.english_ascii && !text.is_ascii() {
                        report.finding("source-non-ascii", &name);
                    }
                }
                Err(code) => report.gap(code, &name),
            }
        }
    }
}
