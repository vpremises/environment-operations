//! Own collector lifetime and keep stdout bounded in memory rather than on disk.
use serde_json::Value;
use std::{
    io::Read,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
const MAX_OUTPUT: u64 = 16 * 1024 * 1024;

struct Guard(Child);
impl Drop for Guard {
    fn drop(&mut self) {
        // Collectors are trusted local programs, not a sandbox: escaped groups
        // cannot be contained here. Always stop descendants in our own group.
        let _ = Command::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{}", self.0.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub(super) fn run(mut command: Command, seconds: u64) -> Result<(i32, Value), &'static str> {
    let mut child = Guard(
        command
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| "collector-unavailable")?,
    );
    let stdout = child.0.stdout.take().ok_or("collector-unavailable")?;
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout
            .take(MAX_OUTPUT + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut output = None;
    let status = loop {
        if Instant::now() >= deadline {
            return Err("collector-bound-exceeded");
        }
        if output.is_none() {
            match receiver.try_recv() {
                Ok(Ok(bytes)) if bytes.len() as u64 <= MAX_OUTPUT => output = Some(bytes),
                Ok(Ok(_)) => return Err("collector-bound-exceeded"),
                Ok(Err(_)) | Err(mpsc::TryRecvError::Disconnected) => {
                    return Err("collector-invalid-output")
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        match child.0.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(_) => return Err("collector-failed"),
        }
    };
    // Parent exit does not prove its descendants exited or closed stdout.
    drop(child);
    let bytes = match output {
        Some(bytes) => bytes,
        None => receiver
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| "collector-invalid-output")?
            .map_err(|_| "collector-invalid-output")?,
    };
    if bytes.len() as u64 > MAX_OUTPUT {
        return Err("collector-bound-exceeded");
    }
    let value = serde_json::from_slice(&bytes).map_err(|_| "collector-invalid-output")?;
    Ok((status.code().ok_or("collector-failed")?, value))
}
