//! Bounded local observation and mounted-report acquisition for vPremises.

#![forbid(unsafe_code)]
#[cfg(not(target_os = "linux"))]
compile_error!("vpremises-security is supported only on Linux/WSL");

#[cfg(target_os = "linux")]
mod audit;
#[cfg(target_os = "linux")]
pub use audit::{
    audit, initialize_bundle, load_audit_config, read_local_document, AuditConfig,
    BoundaryCollector, Collectors, ContentCollector, NetworkCollector, Tool,
};
mod execution;
mod model;
mod observer;
pub mod oss;
mod path_security;
mod report;
mod security;
mod validation;
pub use security::{security_report, CheckCoverage, SecurityReport};

pub use execution::{
    ExecutionContractError, ExecutionControlKindV1, ExecutionControlRequestV1,
    ExecutionLeaseRequestV1, ExecutionLeaseV1, ExecutionOutcomeV1, ExecutionPlacementV1,
    ExecutionResultV1, ExecutionStateV1, ProjectionReferenceV1, ValidateExecution,
};
pub use model::{
    AllowedRoot, Diagnostic, ExpectedDepartmentDailyReport, MountedSharePointReportReceipt,
    MountedSharePointReportRequest, ObservationLimits, ObservationPolicy, ObservationReport,
    ObservationTotals, ObserverConfig, ReportAcquisitionError, ReportAllowedRoot,
    ReportClassification, ReportRecordMode, ReportRootBase, ReportedPolicy, RootObservation,
};
pub use observer::observe;
pub use report::acquire as acquire_mounted_sharepoint_report;

/// Closed metadata-observer configuration schema.
pub const CONFIG_SCHEMA_VERSION: &str = "vpremises.observer/v1";
/// Closed mounted-report acquisition request schema.
pub const MOUNTED_SHAREPOINT_REQUEST_SCHEMA: &str =
    "vpremises://contracts/mounted-sharepoint-report-request/v1";
/// Content-free mounted-report receipt schema.
pub const MOUNTED_SHAREPOINT_RECEIPT_SCHEMA: &str =
    "vpremises://contracts/mounted-sharepoint-report-receipt/v1";
/// Transport-neutral action execution lease request.
pub const EXECUTION_LEASE_REQUEST_SCHEMA_V1: &str =
    "vpremises://contracts/execution-lease-request/v1";
/// Accepted immutable execution lease.
pub const EXECUTION_LEASE_SCHEMA_V1: &str = "vpremises://contracts/execution-lease/v1";
/// Transport-neutral action execution result.
pub const EXECUTION_RESULT_SCHEMA_V1: &str = "vpremises://contracts/execution-result/v1";
/// Status, cancellation and recovery request for an execution lease.
pub const EXECUTION_CONTROL_SCHEMA_V1: &str = "vpremises://contracts/execution-control/v1";
/// Only report body schema accepted by the mounted acquisition boundary.
pub const DEPARTMENT_DAILY_REPORT_SCHEMA: &str = "estate://operations/department-daily-report/v1";
/// Only mounted report media type accepted by the acquisition boundary.
pub const REPORT_MEDIA_TYPE: &str = "application/json";

pub(crate) const MAX_REPORT_BYTES: u64 = 4 * 1024 * 1024;
pub(crate) const MAX_RELATIVE_FILE_BYTES: usize = 512;
pub(crate) const MAX_RELATIVE_COMPONENTS: usize = 16;
