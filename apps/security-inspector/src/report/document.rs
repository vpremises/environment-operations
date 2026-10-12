//! Report-body validation is split between intrinsic content and requested context.

mod content;
mod expected;

use crate::model::{DepartmentDailyReport, ExpectedDepartmentDailyReport, ReportAcquisitionError};

pub(super) fn validate(bytes: &[u8]) -> Result<DepartmentDailyReport, ReportAcquisitionError> {
    content::validate(bytes)
}

pub(super) fn validate_expected(
    expected: &ExpectedDepartmentDailyReport,
    report: &DepartmentDailyReport,
) -> Result<(), ReportAcquisitionError> {
    expected::validate(expected, report)
}
