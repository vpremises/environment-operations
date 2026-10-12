//! Traversal rechecks canonical ancestry and accounts entries without exposing their names.

mod directory;
mod totals;

use crate::{
    model::{Diagnostic, ObservationLimits, ObservationTotals},
    observer::{diagnostic, ValidatedRoot},
};
use std::fs;
use totals::EntryKind;

pub(super) fn root(
    root: &ValidatedRoot,
    limits: &ObservationLimits,
    global: &mut ObservationTotals,
    local: &mut ObservationTotals,
) -> Result<(), Diagnostic> {
    totals::account(root, limits, global, local, EntryKind::Directory, 0, 0)?;
    let held = directory::root(&root.canonical_path).map_err(|_| {
        diagnostic::create(
            "vpremises.scan.root-open-failed",
            Some(&root.id),
            "observation",
            "the selected directory could not be opened without following links",
        )
    })?;
    let mut pending = vec![(held, 0_u32)];
    while let Some((held, depth)) = pending.pop() {
        let entries = fs::read_dir(directory::path(&held)).map_err(|error| {
            diagnostic::io(
                "vpremises.scan.read-directory-failed",
                &root.id,
                "observation",
                &error,
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                diagnostic::io(
                    "vpremises.scan.read-entry-failed",
                    &root.id,
                    "observation",
                    &error,
                )
            })?;
            inspect_entry(
                root,
                limits,
                global,
                local,
                &mut pending,
                &entry.path(),
                depth + 1,
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn inspect_entry(
    root: &ValidatedRoot,
    limits: &ObservationLimits,
    global: &mut ObservationTotals,
    local: &mut ObservationTotals,
    pending: &mut Vec<(std::fs::File, u32)>,
    path: &std::path::Path,
    depth: u32,
) -> Result<(), Diagnostic> {
    if depth > limits.max_depth {
        return Err(diagnostic::create(
            "vpremises.limit.depth-exceeded",
            Some(&root.id),
            "observation",
            "max_depth was exceeded",
        ));
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        diagnostic::io(
            "vpremises.scan.metadata-failed",
            &root.id,
            "observation",
            &error,
        )
    })?;
    let file_type = metadata.file_type();
    let (kind, bytes) = if crate::path_security::is_link(&metadata) {
        (EntryKind::Symlink, 0)
    } else if file_type.is_dir() {
        if pending.len() >= 256 {
            return Err(diagnostic::create(
                "vpremises.limit.open-directories-exceeded",
                Some(&root.id),
                "observation",
                "the held directory descriptor limit was exceeded",
            ));
        }
        let child = directory::child(path).map_err(|error| {
            diagnostic::io(
                "vpremises.scan.directory-open-failed",
                &root.id,
                "observation",
                &error,
            )
        })?;
        pending.push((child, depth));
        (EntryKind::Directory, 0)
    } else if file_type.is_file() {
        (EntryKind::File, metadata.len())
    } else {
        (EntryKind::Other, 0)
    };
    totals::account(root, limits, global, local, kind, depth, bytes)
}
