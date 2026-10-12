//! Resolve external configuration paths explicitly from the private config directory.
use super::{io, AuditConfig};
use std::path::{Component, Path, PathBuf};

/// Read a physical local document with no-follow ancestor and byte/change checks.
/// # Errors
/// Reject linked, non-regular, changing or oversized input and invalid bounds.
pub fn read_local_document(path: &Path, maximum: u64) -> Result<Vec<u8>, &'static str> {
    if !(1..=4_194_304).contains(&maximum) {
        return Err("input-boundary");
    }
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|_| "input-unavailable")?
            .join(path)
    };
    io::read(&absolute, maximum)
}

/// Load a bounded audit configuration without following linked ancestors.
/// # Errors
/// Reject invalid JSON, unsafe relative paths and non-physical configuration files.
pub fn load_audit_config(path: &Path) -> Result<AuditConfig, &'static str> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|_| "config-unavailable")?
            .join(path)
    };
    let mut config: AuditConfig =
        serde_json::from_slice(&io::read(&path, 1_048_576)?).map_err(|_| "audit-config-invalid")?;
    let base = path.parent().ok_or("config-unavailable")?;
    for root in &mut config.observer.roots {
        resolve(base, &mut root.path)?;
    }
    if let Some(c) = &mut config.collectors.content {
        resolve(base, &mut c.tool.executable)?;
        resolve(base, &mut c.settings)?;
    }
    if let Some(c) = &mut config.collectors.network {
        resolve(base, &mut c.tool.executable)?;
        if let Some(path) = &mut c.baseline {
            resolve(base, path)?;
        }
    }
    if let Some(c) = &mut config.collectors.boundary {
        resolve(base, &mut c.tool.executable)?;
        resolve(base, &mut c.input)?;
    }
    Ok(config)
}
fn resolve(base: &Path, path: &mut PathBuf) -> Result<(), &'static str> {
    if path
        .components()
        .any(|p| matches!(p, Component::ParentDir | Component::Prefix(_)))
    {
        return Err("config-path-invalid");
    }
    if !path.is_absolute() {
        *path = base.join(&*path);
    }
    Ok(())
}
