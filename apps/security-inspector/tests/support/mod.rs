//! Shared fixtures isolate filesystem mutations and construct closed public requests.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
#[cfg(target_os = "linux")]
use vpremises::{
    acquire_mounted_sharepoint_report, ExpectedDepartmentDailyReport,
    MountedSharePointReportReceipt, MountedSharePointReportRequest, ReportAcquisitionError,
    ReportAllowedRoot, ReportClassification, ReportRecordMode, ReportRootBase,
    MOUNTED_SHAREPOINT_REQUEST_SCHEMA,
};
use vpremises::{
    AllowedRoot, ObservationLimits, ObservationPolicy, ObserverConfig, CONFIG_SCHEMA_VERSION,
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

pub struct TempDirectory(pub PathBuf);

impl TempDirectory {
    pub fn create() -> Self {
        let suffix = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("vpremises-test-{}-{suffix}", std::process::id()));
        fs::create_dir(&path).expect("create test directory");
        Self(path)
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn observer_config(path: &Path) -> ObserverConfig {
    ObserverConfig {
        schema_version: CONFIG_SCHEMA_VERSION.to_owned(),
        roots: vec![AllowedRoot {
            id: "workspace".to_owned(),
            path: path.to_owned(),
        }],
        limits: ObservationLimits {
            max_depth: 8,
            max_entries: 100,
            max_total_bytes: 10_000,
        },
        policy: ObservationPolicy {
            metadata_only: true,
            follow_symlinks: false,
        },
    }
}

#[cfg(target_os = "linux")]
pub fn report_request(root: &Path, relative_file: &str) -> MountedSharePointReportRequest {
    MountedSharePointReportRequest {
        schema: MOUNTED_SHAREPOINT_REQUEST_SCHEMA.to_owned(),
        request_id: "request-daily-20260724".to_owned(),
        correlation_id: "case-daily-20260724".to_owned(),
        artifact_id: "internal-platform-report-department-daily-20260724".to_owned(),
        expected_report: ExpectedDepartmentDailyReport {
            record_mode: ReportRecordMode::SimulationSeed,
            reporting_date: "2026-07-24".to_owned(),
            headquarters_id: "software-platform-oss".to_owned(),
            department_id: "internal-platform".to_owned(),
            reporting_team_id: "internal-platform-team-01".to_owned(),
            owner_account_id: "internal-platform-role-01-01".to_owned(),
            classification: ReportClassification::InternalConfidential,
            customer_data: false,
        },
        allowlisted_root: ReportAllowedRoot {
            id: "mounted-sharepoint-reports".to_owned(),
            base: ReportRootBase::RequestDirectory,
            relative_path: PathBuf::from(root.file_name().expect("root name")),
        },
        relative_file: relative_file.to_owned(),
        max_bytes: 4096,
    }
}

#[cfg(target_os = "linux")]
pub fn acquire(
    request: &MountedSharePointReportRequest,
    root: &Path,
) -> Result<MountedSharePointReportReceipt, ReportAcquisitionError> {
    acquire_mounted_sharepoint_report(request, root.parent().expect("root parent"))
}

#[cfg(target_os = "linux")]
pub fn daily_report(summary: &str) -> String {
    include_str!("../../examples/fixtures/department-daily-report.json").replace(
        "Bounded fixture used to verify mounted report acquisition.",
        summary,
    )
}
