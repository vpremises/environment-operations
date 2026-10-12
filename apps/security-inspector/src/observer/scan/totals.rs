//! Global limits are checked before both global and per-root counters advance.

use crate::{
    model::{Diagnostic, ObservationLimits, ObservationTotals},
    observer::{diagnostic, ValidatedRoot},
};

#[derive(Clone, Copy)]
pub(super) enum EntryKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn account(
    root: &ValidatedRoot,
    limits: &ObservationLimits,
    global: &mut ObservationTotals,
    local: &mut ObservationTotals,
    kind: EntryKind,
    depth: u32,
    file_bytes: u64,
) -> Result<(), Diagnostic> {
    if global.entries_examined.saturating_add(1) > limits.max_entries {
        return Err(diagnostic::create(
            "vpremises.limit.entries-exceeded",
            Some(&root.id),
            "observation",
            "max_entries was exceeded",
        ));
    }
    if global.total_file_bytes.saturating_add(file_bytes) > limits.max_total_bytes {
        return Err(diagnostic::create(
            "vpremises.limit.bytes-exceeded",
            Some(&root.id),
            "observation",
            "max_total_bytes was exceeded",
        ));
    }
    for totals in [&mut *global, local] {
        totals.entries_examined += 1;
        totals.total_file_bytes += file_bytes;
        totals.max_depth_reached = totals.max_depth_reached.max(depth);
        match kind {
            EntryKind::Directory => totals.directories += 1,
            EntryKind::File => totals.files += 1,
            EntryKind::Symlink => totals.symlinks_skipped += 1,
            EntryKind::Other => totals.other_entries += 1,
        }
    }
    Ok(())
}
