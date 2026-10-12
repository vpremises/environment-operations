//! Initialize private settings from an explicitly verified extracted release bundle.
use super::io;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, os::fd::AsRawFd, os::unix::fs::DirBuilderExt, path::Path};

/// Create new private audit settings without overwriting an existing directory.
/// # Errors
/// Reject missing collectors, modified bundle binaries and unsafe output paths.
pub fn initialize_bundle(bundle: &Path, output: &Path, root: &Path) -> Result<(), &'static str> {
    if !output.is_absolute() || !root.is_absolute() || output.starts_with(root) {
        return Err("setup-path-invalid");
    }
    let root_handle = io::open(root)?;
    if !root_handle
        .metadata()
        .map_err(|_| "root-unavailable")?
        .is_dir()
    {
        return Err("setup-root-invalid");
    }
    let manifest: Value =
        serde_json::from_slice(&io::read(&bundle.join("collectors.json"), 65536)?)
            .map_err(|_| "bundle-manifest-invalid")?;
    if manifest["schema"] != "vpremises-security/collectors/v1" {
        return Err("bundle-manifest-invalid");
    }
    let mut tools = serde_json::Map::new();
    for name in [
        "zixcel-repository-security",
        "crowsi-host-network-sensor",
        "crowsi-boundary-monitor",
        "gitleaks",
    ] {
        let path = bundle.join("tools").join(name);
        let bytes = io::read(&path, 64 * 1024 * 1024)?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        if !bytes.starts_with(b"\x7fELF") || manifest["tools"][name]["sha256"] != digest {
            return Err("bundle-tool-pin-mismatch");
        }
        tools.insert(
            name.into(),
            json!({"executable":path,"sha256":digest,"timeout_seconds":120}),
        );
    }
    let settings = json!({"repository_id":"selected-mount",
        "gitleaks":{"path":tools["gitleaks"]["executable"],"sha256":tools["gitleaks"]["sha256"]},
        "private_dictionary_file":"dictionary.json"});
    let audit = json!({"schema":"vpremises-security/audit/v1","collector_budget_seconds":300,
        "observer":{"schema_version":"vpremises.observer/v1","roots":[{"id":"selected-mount","path":root}],
            "limits":{"max_depth":16,"max_entries":20_000,"max_total_bytes":268_435_456},
            "policy":{"metadata_only":true,"follow_symlinks":false}},
        "collectors":{"content":{"tool":tools["zixcel-repository-security"],"settings":"detection.json"},
            "network":{"tool":tools["crowsi-host-network-sensor"],"baseline":null},"boundary":null}});
    let parent = io::open(output.parent().ok_or("setup-path-invalid")?)?;
    let name = output.file_name().ok_or("setup-path-invalid")?;
    let anchored =
        std::path::PathBuf::from(format!("/proc/self/fd/{}", parent.as_raw_fd())).join(name);
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&anchored)
        .map_err(|_| "setup-output-exists-or-unavailable")?;
    let directory = io::open(output)?;
    let held = std::path::PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()));
    for (name, value) in [
        ("audit.json", audit),
        ("detection.json", settings),
        ("dictionary.json", json!([])),
    ] {
        let mut file = io::output(&held.join(name))?;
        file.write_all(
            &serde_json::to_vec_pretty(&value).map_err(|_| "setup-serialization-failed")?,
        )
        .and_then(|()| file.sync_all())
        .map_err(|_| "setup-write-failed")?;
    }
    Ok(())
}
