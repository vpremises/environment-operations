//! Preserve exact reviewed exceptions without ignoring arbitrary private data.
use super::OssPolicy;

pub(super) fn relative(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains(['\\', '\0', ':'])
        && name
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}

pub(super) fn forbidden(name: &str, policy: &OssPolicy) -> bool {
    let parts: Vec<_> = name.split('/').collect();
    let first = parts[0];
    let roots = [
        "registration",
        "state",
        ".local",
        ".state",
        ".data",
        "secrets",
        "credentials",
        "target",
        "node_modules",
        "dist",
        "build",
        "coverage",
        ".cache",
        ".pytest_cache",
        ".ruff_cache",
        ".mypy_cache",
        ".tox",
        ".nox",
        "htmlcov",
    ];
    let vendor = policy.allowed_vendor_archives.iter().any(|p| p == name);
    let generated = policy
        .forbidden_extensions
        .iter()
        .any(|s| name.ends_with(s))
        || roots.contains(&first)
        || parts.contains(&"node_modules")
        || parts.contains(&"__pycache__")
        || ([".pyc", ".tsbuildinfo", ".tgz"]
            .iter()
            .any(|s| name.ends_with(s))
            && !vendor)
        || (parts.len() == 1
            && [".tar.gz", ".zip", ".whl", ".crate", ".profraw", ".profdata"]
                .iter()
                .any(|s| name.ends_with(s)))
        || (first.starts_with(".env")
            && ![".env.example", ".env.sample", ".env.template"].contains(&first));
    generated && !policy.allowed_source_files.iter().any(|p| p == name)
}

pub(super) fn archive_forbidden(name: &str) -> bool {
    let parts: Vec<_> = name.split('/').collect();
    let leaf = parts.last().copied().unwrap_or_default();
    parts.iter().any(|p| {
        [
            "registration",
            "state",
            ".state",
            ".data",
            "secrets",
            "credentials",
            ".git",
            "node_modules",
            "target",
            ".output",
            ".nuxt",
        ]
        .contains(p)
    }) || leaf.starts_with(".env")
        || [
            ".tgz", ".crate", ".p12", ".pfx", ".pem", ".key", ".sqlite", ".db",
        ]
        .iter()
        .any(|s| leaf.ends_with(s))
}
