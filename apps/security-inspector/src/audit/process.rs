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
    let deadline = Instant::now() + Duration::from_secs(seconds);
    command.stdout(Stdio::piped());
    let mut child = Guard(spawn(&mut command, deadline)?);
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

// A concurrent fork may briefly inherit the private copy's writable descriptor.
// ETXTBSY means exec never began: only this pre-exec refusal can be retried.
// Once spawn returns a Child, every outcome follows the single existing Guard.
fn spawn(command: &mut Command, deadline: Instant) -> Result<Child, &'static str> {
    let retry_end = (Instant::now() + Duration::from_millis(100)).min(deadline);
    let mut reported = false;
    loop {
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(error)
                if error.kind() == std::io::ErrorKind::ExecutableFileBusy
                    && Instant::now() < retry_end =>
            {
                if !reported {
                    eprintln!("collector pre-exec file busy; execution not started; bounded admission retry");
                    reported = true;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(error) => {
                eprintln!(
                    "collector spawn refused: kind={:?}, errno={:?}",
                    error.kind(),
                    error.raw_os_error()
                );
                return Err("collector-unavailable");
            }
        }
    }
}
