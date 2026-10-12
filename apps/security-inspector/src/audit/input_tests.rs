//! Configuration and baseline reads reject linked ancestors and special devices.
use super::super::{budget::Budget, config::Tool, io, load::read_local_document};
use std::{os::unix::fs::symlink, time::Duration};
#[test]
fn document_reader_rejects_linked_ancestors_devices_and_oversized_files() {
    let scratch = io::Scratch::create().unwrap();
    let physical = scratch.0.join("physical");
    std::fs::create_dir(&physical).unwrap();
    std::fs::write(physical.join("document.json"), b"{}").unwrap();
    let linked = scratch.0.join("linked");
    symlink(&physical, &linked).unwrap();
    assert_eq!(
        read_local_document(&physical.join("document.json"), 2).unwrap(),
        b"{}"
    );
    assert!(read_local_document(&linked.join("document.json"), 2).is_err());
    assert!(read_local_document(&physical.join("document.json"), 1).is_err());
    assert!(io::read(std::path::Path::new("/dev/zero"), 1024).is_err());
}
#[test]
fn one_second_budget_is_usable_then_exhausted_and_invalid_timeouts_stay_invalid() {
    let budget = Budget::new(1).unwrap();
    let mut tool = Tool {
        executable: "/usr/bin/false".into(),
        sha256: "0".repeat(64),
        timeout_seconds: 600,
    };
    assert_eq!(budget.tool(&tool).unwrap().timeout_seconds, 1);
    tool.timeout_seconds = 601;
    assert_eq!(budget.tool(&tool).unwrap_err(), "invalid-timeout");
    tool.timeout_seconds = 600;
    std::thread::sleep(Duration::from_millis(1050));
    assert_eq!(
        budget.tool(&tool).unwrap_err(),
        "collector-budget-exhausted"
    );
    assert!(Budget::new(0).is_err());
    assert!(Budget::new(3601).is_err());
}
