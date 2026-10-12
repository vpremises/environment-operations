#![cfg(target_os = "linux")]

//! Boundary tests reject traversal, identity mismatch, symlinks, and explicit size overflow.

use crate::support::{acquire, daily_report, report_request, TempDirectory};
use std::{fs, path::PathBuf};

#[cfg(target_os = "linux")]
#[test]
fn rejects_unsafe_relative_files_and_oversized_reports() {
    let root = TempDirectory::create();
    fs::write(root.0.join("report.json"), daily_report("bounded")).expect("report");
    for relative in [
        "../report.json",
        r"..\report.json",
        "/tmp/report.json",
        "daily%2freport.json",
        "report.txt",
    ] {
        let error = acquire(&report_request(&root.0, relative), &root.0)
            .expect_err("unsafe file must fail");
        assert_eq!(error.code, "vpremises.report.relative-file-invalid");
    }
    let mut request = report_request(&root.0, "report.json");
    request.max_bytes = 4;
    assert_eq!(
        acquire(&request, &root.0).expect_err("size limit").code,
        "vpremises.report.size-exceeded"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_report_and_request_identity_mismatch() {
    let root = TempDirectory::create();
    fs::write(root.0.join("report.json"), daily_report("bounded")).expect("report");
    let mut request = report_request(&root.0, "report.json");
    request.artifact_id = "another-operational-report".to_owned();
    assert_eq!(
        acquire(&request, &root.0)
            .expect_err("identity mismatch")
            .code,
        "vpremises.report.artifact-id-mismatch"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_absolute_and_traversing_allowlisted_roots() {
    let root = TempDirectory::create();
    fs::write(root.0.join("report.json"), daily_report("bounded")).expect("report");
    for unsafe_root in [
        PathBuf::from("/tmp"),
        PathBuf::from("../outside"),
        PathBuf::from(r"..\outside"),
    ] {
        let mut request = report_request(&root.0, "report.json");
        request.allowlisted_root.relative_path = unsafe_root;
        assert_eq!(
            acquire(&request, &root.0).expect_err("unsafe root").code,
            "vpremises.report.root-relative-path-invalid"
        );
    }
}

#[cfg(all(target_os = "linux", unix))]
#[test]
fn rejects_symlink_in_report_path() {
    use std::os::unix::fs::symlink;

    let root = TempDirectory::create();
    let outside = TempDirectory::create();
    fs::write(outside.0.join("report.json"), daily_report("outside")).expect("outside report");
    symlink(&outside.0, root.0.join("linked")).expect("symlink");
    let error = acquire(&report_request(&root.0, "linked/report.json"), &root.0)
        .expect_err("symlink must fail");
    assert_eq!(error.code, "vpremises.report.symlink-forbidden");
}

#[cfg(all(target_os = "linux", unix))]
#[test]
fn rejects_symlinked_report_file_and_allowlisted_root() {
    use std::os::unix::fs::symlink;
    use vpremises::acquire_mounted_sharepoint_report;

    let root = TempDirectory::create();
    let outside = TempDirectory::create();
    fs::write(outside.0.join("report.json"), daily_report("outside")).expect("outside report");
    symlink(outside.0.join("report.json"), root.0.join("report.json")).expect("file symlink");
    let error = acquire(&report_request(&root.0, "report.json"), &root.0)
        .expect_err("final symlink must fail");
    assert_eq!(error.code, "vpremises.report.symlink-forbidden");

    let base = TempDirectory::create();
    symlink(&outside.0, base.0.join("linked")).expect("root symlink");
    let mut request = report_request(&outside.0, "report.json");
    request.allowlisted_root.relative_path = PathBuf::from("linked");
    let error = acquire_mounted_sharepoint_report(&request, &base.0)
        .expect_err("allowlisted root symlink must fail");
    assert_eq!(error.code, "vpremises.report.root-invalid");
}
