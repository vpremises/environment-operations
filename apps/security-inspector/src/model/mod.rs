//! Public contracts keep general observation separate from mounted-report acquisition.

mod observation;
mod observation_result;
pub use observation_result::{
    ObservationReport, ObservationTotals, ReportedPolicy, RootObservation,
};
mod receipt;
mod report;
pub use receipt::{MountedSharePointReportReceipt, ReportAcquisitionError};
pub(crate) mod report_document;

pub use observation::{
    AllowedRoot, Diagnostic, ObservationLimits, ObservationPolicy, ObserverConfig,
};
pub use report::{
    ExpectedDepartmentDailyReport, MountedSharePointReportRequest, ReportAllowedRoot,
    ReportClassification, ReportRecordMode, ReportRootBase,
};
pub(crate) use report_document::DepartmentDailyReport;
