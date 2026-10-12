//! Observer tests prove aggregate disclosure, limit enforcement, and no-follow behavior.

use crate::support::{observer_config, TempDirectory};
use std::{fs, io::Write, path::Path};
use vpremises::observe;

#[test]
fn observes_aggregate_metadata_without_names_or_content() {
    let root = TempDirectory::create();
    fs::create_dir(root.0.join("nested")).expect("nested directory");
    let mut file = fs::File::create(root.0.join("nested/private-name.txt")).expect("file");
    file.write_all(b"secret content").expect("write file");
    let report = observe(&observer_config(&root.0));
    assert!(report.ok);
    assert_eq!(
        (
            report.totals.directories,
            report.totals.files,
            report.totals.total_file_bytes,
        ),
        (2, 1, 14)
    );
    let serialized = serde_json::to_string(&report).expect("serialize report");
    for secret in [
        "private-name",
        "secret content",
        root.0.to_str().expect("path"),
    ] {
        assert!(!serialized.contains(secret));
    }
}

#[test]
fn enforces_global_entry_limit() {
    let root = TempDirectory::create();
    fs::File::create(root.0.join("one")).expect("file");
    let mut config = observer_config(&root.0);
    config.limits.max_entries = 1;
    let report = observe(&config);
    assert!(!report.ok);
    assert_eq!(
        report.diagnostics[0].code,
        "vpremises.limit.entries-exceeded"
    );
}

#[test]
fn rejects_relative_and_system_roots() {
    let relative = observe(&observer_config(Path::new("relative")));
    assert!(relative
        .diagnostics
        .iter()
        .any(|item| item.code == "vpremises.root.not-absolute"));
    let root = TempDirectory::create();
    let system = observe(&observer_config(
        root.0.ancestors().last().expect("filesystem root"),
    ));
    assert!(system
        .diagnostics
        .iter()
        .any(|item| item.code == "vpremises.root.system-root-forbidden"));
}

#[cfg(unix)]
#[test]
fn counts_but_never_follows_symlinks() {
    use std::os::unix::fs::symlink;

    let root = TempDirectory::create();
    symlink("/", root.0.join("outside")).expect("symlink");
    let report = observe(&observer_config(&root.0));
    assert!(report.ok);
    assert_eq!(report.totals.symlinks_skipped, 1);
    assert_eq!(report.totals.directories, 1);
}

#[cfg(unix)]
#[test]
fn rejects_a_symlink_as_the_allowed_root() {
    use std::os::unix::fs::symlink;

    let target = TempDirectory::create();
    let parent = TempDirectory::create();
    let linked = parent.0.join("linked-root");
    symlink(&target.0, &linked).expect("root symlink");
    let report = observe(&observer_config(&linked));
    assert!(report
        .diagnostics
        .iter()
        .any(|item| item.code == "vpremises.root.symlink-forbidden"));
}

#[cfg(unix)]
#[test]
fn metadata_observation_does_not_require_file_content_access() {
    use std::os::unix::fs::PermissionsExt;

    let root = TempDirectory::create();
    let file = root.0.join("unreadable");
    fs::write(&file, b"private bytes").expect("file");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o0)).expect("permissions");
    let report = observe(&observer_config(&root.0));
    assert!(report.ok, "{:?}", report.diagnostics);
    assert_eq!(report.totals.files, 1);
    assert_eq!(report.totals.total_file_bytes, 13);
}

#[test]
fn unicode_and_space_names_remain_private_metadata() {
    let root = TempDirectory::create();
    let directory = root.0.join("space \u{4e2d}");
    fs::create_dir(&directory).expect("Unicode directory");
    fs::write(directory.join("private.txt"), b"synthetic").expect("file");
    let report = observe(&observer_config(&root.0));
    assert!(report.ok);
    assert_eq!(report.totals.files, 1);
    let json = serde_json::to_string(&report).expect("aggregate JSON");
    assert!(!json.contains("space \u{4e2d}"));
    assert!(!json.contains("private.txt"));
}
