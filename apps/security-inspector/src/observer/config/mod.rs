//! Configuration validation separates global policy from filesystem-root resolution.

mod policy;
mod roots;

use crate::{
    model::{Diagnostic, ObserverConfig},
    observer::ValidatedRoot,
};

pub(super) fn validate(config: &ObserverConfig) -> (Vec<ValidatedRoot>, Vec<Diagnostic>) {
    let mut diagnostics = policy::validate(config);
    if !diagnostics.is_empty() {
        return (Vec::new(), diagnostics);
    }
    let mut validated_roots = roots::validate(&config.roots, &mut diagnostics);
    roots::reject_overlaps(&mut validated_roots, &mut diagnostics);
    (validated_roots, diagnostics)
}
