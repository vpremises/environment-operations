//! Stable, path-free acquisition errors are safe to return across the package boundary.

use crate::model::ReportAcquisitionError;

pub(super) const fn create(
    code: &'static str,
    field: &'static str,
    message: &'static str,
) -> ReportAcquisitionError {
    ReportAcquisitionError {
        code,
        field,
        message,
    }
}
