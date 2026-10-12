//! Enumerate canonical organization checkouts, not stale copies or build trees.
use super::workspace_report::{RepositoryResult, WorkspaceReport};
use super::{inspect_exclusions, inspect_repository, OssReport};
use std::path::Path;

/// Inspect each physical Git checkout directly under organizations/owner/repository.
/// # Errors
/// Invalid scope or inaccessible inventory cannot produce an empty successful audit.
pub fn inspect_workspace(root: &Path, gate: &str) -> Result<WorkspaceReport, &'static str> {
    if !["repository", "exclusions"].contains(&gate) {
        return Err("workspace-gate-invalid");
    }
    let root = super::root(root)?;
    let organizations = root.join("organizations");
    let mut repositories = vec![];
    for owner in directories(&organizations)? {
        for repository in directories(&owner)? {
            if !repository
                .join(".git")
                .try_exists()
                .map_err(|_| "inventory-unavailable")?
            {
                continue;
            }
            let name = repository
                .strip_prefix(&root)
                .map_err(|_| "inventory-invalid")?
                .to_str()
                .ok_or("inventory-encoding")?
                .to_owned();
            let result = if gate == "repository" {
                inspect_repository(&repository, None)
            } else {
                inspect_exclusions(&repository, None)
            };
            let report = result.unwrap_or_else(|code| {
                let mut report = OssReport::new(if gate == "repository" {
                    "repository"
                } else {
                    "exclusions"
                });
                report.gap(code, "");
                report.finish()
            });
            repositories.push(RepositoryResult {
                repository: name,
                report,
            });
            if repositories.len() > 10_000 {
                return Err("inventory-budget");
            }
        }
    }
    if repositories.is_empty() {
        return Err("inventory-empty");
    }
    repositories.sort_by(|a, b| a.repository.cmp(&b.repository));
    let status = if repositories.iter().any(|r| r.report.exit_code() == 2) {
        "incomplete"
    } else if repositories.iter().any(|r| r.report.exit_code() == 3) {
        "findings"
    } else {
        "passed"
    };
    Ok(WorkspaceReport {
        schema: "vpremises-security/oss-workspace/v1",
        status,
        repositories,
        external_actions: false,
    })
}
fn directories(root: &Path) -> Result<Vec<std::path::PathBuf>, &'static str> {
    super::root(root)?;
    let mut output = vec![];
    for entry in std::fs::read_dir(root).map_err(|_| "inventory-unavailable")? {
        let entry = entry.map_err(|_| "inventory-unavailable")?;
        let kind = entry.file_type().map_err(|_| "inventory-unavailable")?;
        if kind.is_symlink() {
            return Err("inventory-symlink");
        }
        if kind.is_dir() {
            output.push(entry.path());
        }
        if output.len() > 10_000 {
            return Err("inventory-budget");
        }
    }
    Ok(output)
}
