//! Git ignore checks use a scratch index, never stage or modify selected sources.
use super::{git, probes, OssReport};
use std::path::Path;

/// Check repository-only ignores, visible generated files and tracked ignored files.
/// # Errors
/// Reject inaccessible roots or invalid policy rather than borrowing another index.
pub fn inspect_exclusions(
    root: &Path,
    policy_path: Option<&Path>,
) -> Result<OssReport, &'static str> {
    let root = super::root(root)?;
    git::validate_root(&root)?;
    let policy = if policy_path.is_none()
        && !root
            .join(".github/oss-policy.json")
            .try_exists()
            .map_err(|_| "policy-unavailable")?
    {
        super::OssPolicy {
            version: 1,
            license: "Apache-2.0".into(),
            allowed_vendor_archives: vec![],
            allowed_source_files: vec![],
            forbidden_extensions: vec![],
            rust_source: None,
        }
    } else {
        super::policy(&root, policy_path)?
    };
    let scratch = crate::audit::io::Scratch::create()?;
    git::run(&scratch.0, &["init", "--quiet"], None)?;
    let dir = scratch.0.join(".git");
    let dir = dir.to_str().ok_or("root-encoding")?;
    let tree = root.to_str().ok_or("root-encoding")?;
    let mut report = OssReport::new("exclusions");
    if super::document::read(&root.join(".gitignore"), 4_194_304).is_err() {
        report.finding("gitignore-missing", ".gitignore");
    }
    let arguments = [
        "--git-dir",
        dir,
        "--work-tree",
        tree,
        "check-ignore",
        "--no-index",
        "-z",
        "--stdin",
    ];
    let input = probes::EXCLUDED
        .iter()
        .chain(probes::SOURCE)
        .copied()
        .collect::<Vec<_>>()
        .join("\0")
        + "\0";
    let bytes = git::run(&root, &arguments, Some(input.as_bytes()))?;
    let ignored: std::collections::BTreeSet<_> = std::str::from_utf8(&bytes)
        .map_err(|_| "git-path-encoding")?
        .split('\0')
        .collect();
    for name in probes::EXCLUDED {
        if !ignored.contains(name) {
            report.finding("missing-exclusion", name);
        }
    }
    for name in probes::SOURCE {
        if ignored.contains(name) {
            report.finding("source-overexcluded", name);
        }
    }
    for name in git::names(
        &root,
        &[
            "--git-dir",
            dir,
            "--work-tree",
            tree,
            "ls-files",
            "--others",
            "--exclude-standard",
            "-z",
        ],
    )? {
        let generated = name
            .split('/')
            .any(|p| ["node_modules", "target", "__pycache__"].contains(&p))
            || name.split('/').next().is_some_and(|p| {
                [
                    ".nuxt",
                    ".output",
                    "dist",
                    "build",
                    ".cache",
                    "coverage",
                    ".pytest_cache",
                ]
                .contains(&p)
            })
            || [".pyc", ".tsbuildinfo", ".tgz", ".tar.gz", ".whl", ".crate"]
                .iter()
                .any(|s| name.ends_with(s));
        if generated && !policy.allowed_source_files.contains(&name) {
            report.finding("visible-generated-file", &name);
        }
    }
    for name in git::names(&root, &["ls-files", "-ci", "--exclude-standard", "-z"])? {
        report.finding("tracked-ignored-file", &name);
    }
    Ok(report.finish())
}
