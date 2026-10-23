//! Real kernel pre-exec refusal and once-only started child acceptance.
use super::super::{io::Scratch, process};
use std::{
    fs::{self, OpenOptions},
    os::unix::{fs::PermissionsExt, process::CommandExt},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn busy_fixture(scratch: &Scratch) -> (Command, std::fs::File) {
    let path = scratch.0.join("printf");
    fs::copy("/usr/bin/printf", &path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    let writer = OpenOptions::new().write(true).open(&path).unwrap();
    let mut command = Command::new(&path);
    command.arg("{}").process_group(0).stderr(Stdio::null());
    (command, writer)
}

#[test]
fn preexec_busy_can_recover_only_before_a_child_started() {
    let scratch = Scratch::create().unwrap();
    let (command, writer) = busy_fixture(&scratch);
    let closer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        drop(writer);
    });
    let (code, value) = process::run(command, 1).unwrap();
    closer.join().unwrap();
    assert_eq!(code, 0);
    assert_eq!(value, serde_json::json!({}));
}

#[test]
fn persistent_preexec_busy_has_a_finite_admission_budget() {
    let scratch = Scratch::create().unwrap();
    let (command, writer) = busy_fixture(&scratch);
    let start = Instant::now();
    assert_eq!(
        process::run(command, 1).unwrap_err(),
        "collector-unavailable"
    );
    assert!(start.elapsed() < Duration::from_secs(1));
    drop(writer);
}

#[test]
fn a_started_nonzero_child_is_never_replayed() {
    let scratch = Scratch::create().unwrap();
    let count = scratch.0.join("count");
    let mut command = Command::new("/usr/bin/bash");
    command
        .args(["-c", r#"printf x >> "$1"; printf '{}'; exit 7"#, "fixture"])
        .arg(&count)
        .process_group(0)
        .stderr(Stdio::null());
    let (code, _) = process::run(command, 1).unwrap();
    assert_eq!(code, 7);
    assert_eq!(fs::read(&count).unwrap(), b"x");
}
