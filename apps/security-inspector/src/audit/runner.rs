//! Execute only pinned ELF collectors with private, bounded output and a deadline.
use super::{
    config::Tool,
    io::{self, Scratch},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    ffi::OsStr,
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::{fs::PermissionsExt, process::CommandExt},
    },
    path::Path,
    process::{Command, Stdio},
};

pub(super) fn run(
    tool: &Tool,
    args: &[&OsStr],
    input: Option<&Path>,
    scratch: &Scratch,
) -> Result<(i32, Value), &'static str> {
    if !(1..=600).contains(&tool.timeout_seconds) {
        return Err("invalid-timeout");
    }
    let executable = io::open(&tool.executable)?;
    if !executable
        .metadata()
        .map_err(|_| "tool-unavailable")?
        .is_file()
    {
        return Err("tool-unavailable");
    }
    let mut bytes = Vec::new();
    (&executable)
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "tool-unavailable")?;
    if bytes.len() > 64 * 1024 * 1024
        || !bytes.starts_with(b"\x7fELF")
        || format!("{:x}", Sha256::digest(&bytes)) != tool.sha256
    {
        return Err("tool-pin-mismatch");
    }
    let copy = scratch.0.join("collector");
    io::output(&copy)?
        .write_all(&bytes)
        .map_err(|_| "tool-copy-failed")?;
    std::fs::set_permissions(&copy, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| "tool-copy-failed")?;
    let executable = io::open(&copy)?;
    let stdin = match input {
        Some(path) => Stdio::from(io::open(path)?),
        None => Stdio::null(),
    };
    // Execute the held descriptor so path replacement cannot select different bytes.
    let mut command = Command::new(format!("/proc/self/fd/{}", executable.as_raw_fd()));
    command
        .args(args)
        .process_group(0)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("HOME", &scratch.0)
        .env("TMPDIR", &scratch.0)
        .current_dir(&scratch.0)
        .stdin(stdin)
        .stderr(Stdio::null());
    super::process::run(command, tool.timeout_seconds)
}
