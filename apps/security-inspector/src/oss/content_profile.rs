//! The native Gitleaks policy preserves the publication marker acceptance cases.
use super::{document, OssReport};
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) fn validate(settings: &Path, report: &mut OssReport) -> Result<(), &'static str> {
    let value = document::load(settings, report)?;
    let profile = value["gitleaks"]["config_file"]
        .as_str()
        .ok_or("archive-content-profile-required")?;
    let profile = settings
        .parent()
        .ok_or("archive-content-profile-invalid")?
        .join(profile);
    let bytes = document::read(&profile, 1_048_576)?;
    let expected = include_bytes!("../../examples/oss.gitleaks.toml");
    if Sha256::digest(&bytes) != Sha256::digest(expected) {
        return Err("archive-content-profile-mismatch");
    }
    report
        .evidence_sha256
        .push(format!("{:x}", Sha256::digest(bytes)));
    Ok(())
}
