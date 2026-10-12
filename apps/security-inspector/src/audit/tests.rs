//! Process failures and deadlines cannot be translated into passing evidence.
use super::{config::Tool, io::Scratch, runner};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsStr,
    time::{Duration, Instant},
};

fn pinned(path: &str) -> Tool {
    Tool {
        executable: path.into(),
        sha256: format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
        timeout_seconds: 1,
    }
}

#[test]
fn malformed_collector_output_is_rejected() {
    let scratch = Scratch::create().unwrap();
    let error = runner::run(&pinned("/usr/bin/false"), &[], None, &scratch).unwrap_err();
    assert_eq!(error, "collector-invalid-output");
}

#[test]
fn collector_deadline_stops_the_process() {
    let scratch = Scratch::create().unwrap();
    let start = Instant::now();
    let error = runner::run(
        &pinned("/usr/bin/sleep"),
        &[OsStr::new("10")],
        None,
        &scratch,
    )
    .unwrap_err();
    assert_eq!(error, "collector-bound-exceeded");
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[path = "input_tests.rs"]
mod input_tests;
#[path = "process_tests.rs"]
mod process_tests;
#[path = "receipt_tests.rs"]
mod receipt_tests;
