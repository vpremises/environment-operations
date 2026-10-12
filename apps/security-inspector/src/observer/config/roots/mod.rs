//! Allowed roots must resolve to unique, non-overlapping directories below the system root.

use crate::{
    model::{AllowedRoot, Diagnostic},
    observer::{diagnostic, ValidatedRoot},
    validation::valid_identifier,
};
mod resolve;
use resolve::resolve;
use std::collections::BTreeSet;

pub(super) fn validate(
    roots: &[AllowedRoot],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ValidatedRoot> {
    let mut ids = BTreeSet::new();
    let mut validated = Vec::new();
    for root in roots {
        validate_identity(root, &mut ids, diagnostics);
        let Some(canonical_path) = resolve(root, diagnostics) else {
            continue;
        };
        validated.push(ValidatedRoot {
            id: root.id.clone(),
            canonical_path,
        });
    }
    validated
}

pub(super) fn reject_overlaps(roots: &mut [ValidatedRoot], diagnostics: &mut Vec<Diagnostic>) {
    for left_index in 0..roots.len() {
        for right_index in (left_index + 1)..roots.len() {
            let left = &roots[left_index];
            let right = &roots[right_index];
            if left.canonical_path.starts_with(&right.canonical_path)
                || right.canonical_path.starts_with(&left.canonical_path)
            {
                diagnostics.push(diagnostic::create(
                    "vpremises.root.overlap",
                    Some(&left.id),
                    "$.roots",
                    format!("allowed roots '{}' and '{}' overlap", left.id, right.id),
                ));
            }
        }
    }
}

fn validate_identity(
    root: &AllowedRoot,
    ids: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !valid_identifier(&root.id) {
        diagnostics.push(diagnostic::create(
            "vpremises.root.invalid-id",
            Some(&root.id),
            "$.roots[].id",
            "root id must contain only lowercase ASCII letters, digits, '.', '_' or '-'",
        ));
    }
    if !ids.insert(root.id.clone()) {
        diagnostics.push(diagnostic::create(
            "vpremises.root.duplicate-id",
            Some(&root.id),
            "$.roots[].id",
            "root ids must be unique",
        ));
    }
}
