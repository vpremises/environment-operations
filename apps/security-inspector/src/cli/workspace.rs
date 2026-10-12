//! Relative observer roots remain inside the configuration package boundary.

use serde_json::{json, Value};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use vpremises::ObserverConfig;

pub(super) fn load_observer_config(path: &Path) -> Result<ObserverConfig, (u8, Value)> {
    let input = super::file_input::read_utf8(path, 1_048_576).map_err(|message| {
        (
            1,
            json!({
                "schema_version": "vpremises.error/v1",
                "ok": false,
                "code": "vpremises.io.read",
                "message": format!("could not read observer configuration: {message}")
            }),
        )
    })?;
    let mut config: ObserverConfig = serde_json::from_str(&input).map_err(|error| {
        (
            1,
            json!({
                "schema_version": "vpremises.error/v1",
                "ok": false,
                "code": "vpremises.json.decode",
                "message": format!("invalid observer JSON at line {}, column {}", error.line(), error.column())
            }),
        )
    })?;
    let config_root = path
        .canonicalize()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    for root in &mut config.roots {
        if root.path.is_absolute() {
            continue;
        }
        if !bounded_relative(&root.path) {
            return Err((1, root_error(
                "vpremises.root.relative-path-invalid",
                "relative observer roots must be bounded package-relative paths without traversal",
            )));
        }
        let config_root = config_root.as_ref().ok_or_else(|| {
            (
                1,
                root_error(
                    "vpremises.root.config-root-unavailable",
                    "relative observer roots require a regular configuration file",
                ),
            )
        })?;
        root.path = config_root.join(&root.path);
    }
    Ok(config)
}

pub(super) fn find_root(directory: &Path) -> Option<PathBuf> {
    directory
        .ancestors()
        .find(|candidate| {
            fs::symlink_metadata(candidate.join("source-foundation.toml"))
                .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        })
        .map(Path::to_path_buf)
}

fn bounded_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path.components().count() <= 16
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn root_error(code: &str, message: &str) -> Value {
    json!({
        "schema_version": "vpremises.error/v1",
        "ok": false,
        "code": code,
        "message": message
    })
}
