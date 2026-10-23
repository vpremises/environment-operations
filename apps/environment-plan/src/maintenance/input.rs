use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{self, BufRead, IsTerminal, Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};
use vpremises_environment_deployment::{
    Error, MaintenanceContext, MaintenanceObservation, MaintenancePlan, MaintenanceStep,
    ResolvedEnvironment, Result, desired_digest, maintenance_plan_digest,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub id: String,
    pub valid_seconds: u16,
    pub max_runtime_seconds: u16,
    pub steps: Vec<MaintenanceStep>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub plan: MaintenancePlan,
    pub plan_sha256: String,
}

pub fn candidate(path: &str) -> Result<Candidate> {
    let candidate: Candidate = crate::read_json(path)?;
    if maintenance_plan_digest(&candidate.plan)? != candidate.plan_sha256 {
        return Err(Error::new("maintenance candidate digest mismatch"));
    }
    Ok(candidate)
}

pub fn now() -> Result<i64> {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::new("system clock unavailable"))?
            .as_secs(),
    )
    .map_err(|_| Error::new("system clock overflow"))
}

pub fn context(
    resolved: &str,
    source: &str,
    artifact: &str,
    inventory: &str,
) -> Result<MaintenanceContext> {
    let desired: ResolvedEnvironment = crate::read_json(resolved)?;
    let (owner, host) = local_identity()?;
    Ok(MaintenanceContext {
        owner,
        host,
        source_sha256: hash_file(source, 64 * 1024 * 1024)?,
        artifact_sha256: hash_file(artifact, 256 * 1024 * 1024)?,
        inventory_sha256: hash_file(inventory, 16 * 1024 * 1024)?,
        desired_sha256: desired_digest(&desired)?,
    })
}

#[cfg(unix)]
pub fn local_identity() -> Result<(String, String)> {
    let machine = zixcel_bounded_io::read_regular_text("/etc/machine-id", 64)?;
    let machine = machine.trim();
    if machine.len() != 32 || !machine.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::new("local machine identity unavailable"));
    }
    Ok((
        format!("unix-uid:{}", rustix::process::geteuid().as_raw()),
        format!("machine-id-sha256:{:x}", Sha256::digest(machine.as_bytes())),
    ))
}

#[cfg(not(unix))]
pub fn local_identity() -> Result<(String, String)> {
    Err(Error::new(
        "local maintenance host admission is only accepted on Linux/WSL",
    ))
}

pub fn observation(path: &str, now: i64) -> Result<MaintenanceObservation> {
    let digest = hash_file(path, 16 * 1024 * 1024)?;
    Ok(MaintenanceObservation {
        state_sha256: digest.clone(),
        evidence_sha256: digest,
        observed_at: now,
    })
}

fn hash_file(path: &str, limit: u64) -> Result<String> {
    let file = zixcel_bounded_io::open_regular_file(path)?;
    if file.metadata()?.len() > limit {
        return Err(Error::new("maintenance file budget exceeded"));
    }
    let mut limited = file.take(limit + 1);
    let mut bytes = 0u64;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = limited.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes +=
            u64::try_from(count).map_err(|_| Error::new("maintenance file length overflow"))?;
        if bytes > limit {
            return Err(Error::new("maintenance file budget exceeded"));
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn confirm(candidate: &Candidate) -> Result<()> {
    if !io::stdin().is_terminal() {
        return Err(Error::new(
            "local maintenance consent requires an operator terminal; no consent recorded",
        ));
    }
    eprintln!("{}", serde_json::to_string_pretty(candidate)?);
    eprintln!(
        "Record local consent only; no host execution authorized. Type the exact plan SHA256:"
    );
    io::stderr().flush()?;
    let mut answer = String::new();
    io::BufReader::new(io::stdin().lock().take(130)).read_line(&mut answer)?;
    if answer.trim_end_matches(['\r', '\n']) != candidate.plan_sha256 {
        return Err(Error::new("maintenance consent not confirmed"));
    }
    Ok(())
}
