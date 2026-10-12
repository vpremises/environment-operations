//! Resolve local roots without accepting system roots or linked entries.

use crate::{
    model::{AllowedRoot, Diagnostic},
    observer::diagnostic,
};
use std::fs;

pub(super) fn resolve(
    root: &AllowedRoot,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<std::path::PathBuf> {
    if !root.path.is_absolute() {
        diagnostics.push(diagnostic::create(
            "vpremises.root.not-absolute",
            Some(&root.id),
            "$.roots[].path",
            "allowed root must be an absolute path",
        ));
        return None;
    }
    let metadata = fs::symlink_metadata(&root.path)
        .map_err(|error| {
            diagnostics.push(diagnostic::io(
                "vpremises.root.unavailable",
                &root.id,
                "$.roots[].path",
                &error,
            ));
        })
        .ok()?;
    if crate::path_security::is_link(&metadata) || !metadata.is_dir() {
        let (code, message) = if crate::path_security::is_link(&metadata) {
            (
                "vpremises.root.symlink-forbidden",
                "an allowed root cannot itself be a symlink",
            )
        } else {
            (
                "vpremises.root.not-directory",
                "an allowed root must be a directory",
            )
        };
        diagnostics.push(diagnostic::create(
            code,
            Some(&root.id),
            "$.roots[].path",
            message,
        ));
        return None;
    }
    let canonical = root
        .path
        .canonicalize()
        .map_err(|error| {
            diagnostics.push(diagnostic::io(
                "vpremises.root.canonicalize-failed",
                &root.id,
                "$.roots[].path",
                &error,
            ));
        })
        .ok()?;
    if canonical.parent().is_none() {
        diagnostics.push(diagnostic::create(
            "vpremises.root.system-root-forbidden",
            Some(&root.id),
            "$.roots[].path",
            "observing the filesystem root is forbidden",
        ));
        None
    } else {
        Some(canonical)
    }
}
