//! Bounded local Git plumbing disables executable fsmonitor and inherited overrides.
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub(super) fn run(
    root: &Path,
    args: &[&str],
    input: Option<&[u8]>,
) -> Result<Vec<u8>, &'static str> {
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .args([
            "--no-pager",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
        ])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0");
    let mut child = command.spawn().map_err(|_| "git-unavailable")?;
    if let Some(data) = input {
        child
            .stdin
            .take()
            .ok_or("git-input-failed")?
            .write_all(data)
            .map_err(|_| "git-input-failed")?;
    } else {
        drop(child.stdin.take());
    }
    let stdout = child.stdout.take().ok_or("git-output-failed")?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .take(8 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let start = Instant::now();
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status
                    .code()
                    .is_some_and(|c| c == 0 || (c == 1 && args.contains(&"check-ignore")))
                {
                    Ok(())
                } else {
                    Err("git-failed")
                }
            }
            Err(_) => break Err("git-failed"),
            Ok(None) if start.elapsed() > Duration::from_secs(10) => break Err("git-timeout"),
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let bytes = reader
        .join()
        .map_err(|_| "git-output-failed")?
        .map_err(|_| "git-output-failed")?;
    result?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("git-output-budget");
    }
    Ok(bytes)
}

pub(super) fn names(root: &Path, args: &[&str]) -> Result<Vec<String>, &'static str> {
    let bytes = run(root, args, None)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "git-path-encoding")?;
    let names: Vec<_> = text
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect();
    if names.iter().any(|p| !super::paths::relative(p)) {
        return Err("git-path-invalid");
    }
    Ok(names)
}

/// Refuse to borrow the index of an enclosing repository.
pub(super) fn validate_root(root: &Path) -> Result<(), &'static str> {
    let bytes = run(root, &["rev-parse", "--show-toplevel"], None)?;
    let name = std::str::from_utf8(&bytes)
        .map_err(|_| "git-path-encoding")?
        .trim_end();
    if Path::new(name)
        .canonicalize()
        .map_err(|_| "git-root-unavailable")?
        != root
    {
        return Err("git-root-mismatch");
    }
    Ok(())
}
