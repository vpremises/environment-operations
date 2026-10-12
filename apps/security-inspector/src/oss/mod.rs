//! Local OSS gates complement content detection without executing repository code.
mod archive;
mod archive_entries;
mod archive_stream;
mod cargo_manifest;
mod catalog;
mod content;
mod content_profile;
mod document;
mod exclusions;
mod git;
mod manifests;
mod npm_manifest;
mod npm_specifier;
mod package;
mod package_manifest;
mod paths;
mod policy;
mod probes;
mod report;
mod repository;
mod source;
mod workspace;
mod workspace_report;

pub use content::audit_archive_content;
pub use exclusions::inspect_exclusions;
pub use package::inspect_package;
pub use policy::{OssPolicy, RustSource};
pub use report::{OssFinding, OssReport};
pub use repository::inspect_repository;
pub use workspace::inspect_workspace;
pub use workspace_report::{RepositoryResult, WorkspaceReport};

use std::path::{Path, PathBuf};

fn root(path: &Path) -> Result<PathBuf, &'static str> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| "root-unavailable")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("root-not-physical-directory");
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| "root-unavailable")?
            .join(path)
    };
    crate::audit::io::open(&absolute).map_err(|_| "root-boundary")?;
    path.canonicalize().map_err(|_| "root-unavailable")
}

fn policy(root: &Path, path: Option<&Path>) -> Result<OssPolicy, &'static str> {
    let path = path.map_or_else(|| root.join(".github/oss-policy.json"), Path::to_path_buf);
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    if std::fs::symlink_metadata(&path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
        return Err("repository-policy-missing");
    }
    let policy: OssPolicy = serde_json::from_slice(&crate::audit::io::read(&path, 1_048_576)?)
        .map_err(|_| "policy-invalid")?;
    policy.validate()?;
    Ok(policy)
}
