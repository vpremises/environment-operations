//! Workspace inventory rejects empty scopes and records per-repository gaps.
use super::support::{git, write};
use vpremises::oss::inspect_workspace;

#[test]
fn empty_inventory_is_not_passed() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("organizations")).unwrap();
    assert_eq!(
        inspect_workspace(root.path(), "repository").unwrap_err(),
        "inventory-empty"
    );
}
#[test]
fn missing_policy_remains_a_repository_specific_gap() {
    let root = tempfile::tempdir().unwrap();
    let repository = root.path().join("organizations/example/library");
    std::fs::create_dir_all(&repository).unwrap();
    git(&repository, &["init", "--quiet"]);
    let report = inspect_workspace(root.path(), "repository").unwrap();
    assert_eq!(report.exit_code(), 2);
    assert_eq!(report.repositories.len(), 1);
    assert_eq!(
        report.repositories[0].repository,
        "organizations/example/library"
    );
    write(&repository, ".gitignore", "target/\n");
    assert_eq!(
        inspect_workspace(root.path(), "exclusions")
            .unwrap()
            .exit_code(),
        3
    );
}

#[test]
fn linked_inventory_ancestors_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("physical/organizations")).unwrap();
    std::os::unix::fs::symlink(root.path().join("physical"), root.path().join("link")).unwrap();
    assert_eq!(
        inspect_workspace(&root.path().join("link/organizations"), "repository").unwrap_err(),
        "root-boundary"
    );
}
