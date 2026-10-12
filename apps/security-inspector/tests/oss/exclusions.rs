//! Scratch ignore checks preserve the original source index and source visibility.
use super::support::{git, has, repository, write};
use vpremises::oss::inspect_exclusions;

#[test]
fn empty_ignore_and_overexcluded_sources_are_findings() {
    let root = repository();
    write(root.path(), ".gitignore", "src/\n");
    let report = inspect_exclusions(root.path(), None).unwrap();
    assert!(has(&report, "missing-exclusion"));
    assert!(has(&report, "source-overexcluded"));
}
#[test]
fn tracked_ignored_and_visible_generated_files_are_reported() {
    let root = repository();
    write(root.path(), "dist/output.js", "synthetic");
    git(root.path(), &["add", "."]);
    write(root.path(), ".gitignore", "/dist/\n");
    write(root.path(), "node_modules/example/index.js", "synthetic");
    let before = std::fs::read(root.path().join(".git/index")).unwrap();
    let report = inspect_exclusions(root.path(), None).unwrap();
    assert!(has(&report, "tracked-ignored-file"));
    assert!(has(&report, "visible-generated-file"));
    assert_eq!(
        before,
        std::fs::read(root.path().join(".git/index")).unwrap()
    );
}
#[test]
fn fsmonitor_hook_is_disabled_during_read_only_audit() {
    let root = repository();
    write(
        root.path(),
        "watcher",
        "#!/bin/sh\ntouch forbidden-marker\n",
    );
    git(root.path(), &["config", "core.fsmonitor", "./watcher"]);
    let _ = inspect_exclusions(root.path(), None).unwrap();
    assert!(!root.path().join("forbidden-marker").exists());
}
