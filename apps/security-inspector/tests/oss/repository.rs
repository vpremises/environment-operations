//! Policy and Git failures must not become successful community checks.
use super::support::{git, has, repository, write};
use vpremises::oss::inspect_repository;

#[test]
fn valid_community_and_virtual_workspace() {
    let root = repository();
    write(root.path(), "Cargo.toml", "[workspace]\nmembers=[]\n");
    assert_eq!(
        inspect_repository(root.path(), None).unwrap().exit_code(),
        0
    );
}
#[test]
fn missing_document_and_wrong_license() {
    let root = repository();
    std::fs::remove_file(root.path().join("NOTICE")).unwrap();
    write(
        root.path(),
        "package.json",
        r#"{"name":"test","license":"MIT"}"#,
    );
    let report = inspect_repository(root.path(), None).unwrap();
    assert!(has(&report, "required-community-file-missing"));
    assert!(has(&report, "manifest-license-mismatch"));
    assert_eq!(report.exit_code(), 3);
}
#[test]
fn tracked_private_files_and_exact_exception() {
    let root = repository();
    write(root.path(), "state/private.json", "synthetic fixture");
    write(root.path(), "fixture.tgz", "synthetic fixture");
    git(root.path(), &["add", "."]);
    assert!(has(
        &inspect_repository(root.path(), None).unwrap(),
        "tracked-private-or-generated-path"
    ));
    write(
        root.path(),
        ".github/oss-policy.json",
        r#"{"version":1,"license":"Apache-2.0","allowed_source_files":["state/private.json"],"allowed_vendor_archives":["fixture.tgz"]}"#,
    );
    assert_eq!(
        inspect_repository(root.path(), None).unwrap().exit_code(),
        0
    );
}
#[test]
fn malformed_policy_and_manifest_are_not_passes() {
    let root = repository();
    write(root.path(), "package.json", "{unparseable");
    assert_eq!(
        inspect_repository(root.path(), None).unwrap().exit_code(),
        2
    );
    write(
        root.path(),
        ".github/oss-policy.json",
        r#"{"version":1,"license":"MIT","allowed_source_files":["state/*"]}"#,
    );
    assert_eq!(
        inspect_repository(root.path(), None).unwrap_err(),
        "policy-exception-invalid"
    );
}
#[test]
fn nested_directory_cannot_borrow_parent_git_index() {
    let root = repository();
    std::fs::create_dir(root.path().join("nested")).unwrap();
    assert_eq!(
        inspect_repository(&root.path().join("nested"), None).unwrap_err(),
        "git-root-mismatch"
    );
}
#[test]
fn symlinked_required_document_is_incomplete() {
    let root = repository();
    std::fs::remove_file(root.path().join("NOTICE")).unwrap();
    std::os::unix::fs::symlink(root.path().join("LICENSE"), root.path().join("NOTICE")).unwrap();
    assert_eq!(
        inspect_repository(root.path(), None).unwrap().exit_code(),
        2
    );
}
#[test]
fn declared_license_and_rust_layout_are_explicit() {
    let root = repository();
    write(
        root.path(),
        "package.json",
        r#"{"name":"test","license":"MIT"}"#,
    );
    write(
        root.path(),
        ".github/oss-policy.json",
        r#"{"version":1,"license":"MIT","rust_source":{"max_lines":2,"english_ascii":true,"directories":["src"]}}"#,
    );
    write(root.path(), "src/lib.rs", "// one\n// two\n// three\n");
    assert!(has(
        &inspect_repository(root.path(), None).unwrap(),
        "source-line-limit"
    ));
}
