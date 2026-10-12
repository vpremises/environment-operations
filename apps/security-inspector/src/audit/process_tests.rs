//! Exercise real subprocesses, inherited pipes and resource-exhaustion attempts.
use super::super::{config::Tool, io::Scratch, runner};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsStr,
    time::{Duration, Instant},
};

fn tool(path: &str, seconds: u64) -> Tool {
    Tool {
        executable: path.into(),
        sha256: format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
        timeout_seconds: seconds,
    }
}
fn stopped(path: &std::path::Path) {
    let pid = std::fs::read_to_string(path).unwrap();
    for _ in 0..100 {
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid.trim()));
        if stat.is_err() || stat.unwrap().split(") ").nth(1).unwrap().starts_with('Z') {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("collector descendant still running");
}
#[test]
fn normal_parent_exit_reclaims_descendants_and_closes_inherited_stdout() {
    let scratch = Scratch::create().unwrap();
    let pid = scratch.0.join("descendant.pid");
    let (code, value) = runner::run(
        &tool("/usr/bin/bash", 3),
        &[
            OsStr::new("-c"),
            OsStr::new("sleep 30 & echo $! > \"$1\"; printf '{\"ok\":true}'"),
            OsStr::new("fixture"),
            pid.as_os_str(),
        ],
        None,
        &scratch,
    )
    .unwrap();
    assert_eq!(code, 0);
    assert_eq!(value["ok"], true);
    stopped(&pid);
}
#[test]
fn timeout_reclaims_detector_descendants() {
    let scratch = Scratch::create().unwrap();
    let pid = scratch.0.join("descendant.pid");
    let error = runner::run(
        &tool("/usr/bin/bash", 1),
        &[
            OsStr::new("-c"),
            OsStr::new("sleep 30 & echo $! > \"$1\"; wait"),
            OsStr::new("fixture"),
            pid.as_os_str(),
        ],
        None,
        &scratch,
    )
    .unwrap_err();
    assert_eq!(error, "collector-bound-exceeded");
    stopped(&pid);
}
#[test]
fn unbounded_stdout_is_stopped_without_a_raw_output_file() {
    let scratch = Scratch::create().unwrap();
    let start = Instant::now();
    let error = runner::run(&tool("/usr/bin/yes", 5), &[], None, &scratch).unwrap_err();
    assert_eq!(error, "collector-bound-exceeded");
    assert!(start.elapsed() < Duration::from_secs(4));
    assert!(!scratch.0.join("stdout.json").exists());
}
