//! Validate gzip EOF and CRC before inspecting TAR headers; trailing streams are rejected.
use std::io::Read;

pub(super) fn expand(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut gzip = flate2::bufread::GzDecoder::new(bytes);
    let mut output = Vec::new();
    gzip.by_ref()
        .take(100_000_001)
        .read_to_end(&mut output)
        .map_err(|_| "archive-invalid")?;
    if output.len() > 100_000_000 {
        return Err("archive-budget");
    }
    if !gzip.into_inner().is_empty() || output.len() < 1024 || output.len() % 512 != 0 {
        return Err("archive-invalid");
    }
    Ok(output)
}
