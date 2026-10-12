//! Receipt identity commits to opaque request fields and the exact report digest.

use crate::{
    model::{
        DepartmentDailyReport, MountedSharePointReportReceipt, MountedSharePointReportRequest,
        ReportClassification,
    },
    DEPARTMENT_DAILY_REPORT_SCHEMA, MOUNTED_SHAREPOINT_RECEIPT_SCHEMA, REPORT_MEDIA_TYPE,
};
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub(super) fn build(
    request: &MountedSharePointReportRequest,
    report: DepartmentDailyReport,
    digest_sha256: String,
    size_bytes: u64,
) -> MountedSharePointReportReceipt {
    MountedSharePointReportReceipt {
        schema: MOUNTED_SHAREPOINT_RECEIPT_SCHEMA,
        request_id: request.request_id.clone(),
        receipt_id: receipt_id(request, &digest_sha256),
        correlation_id: request.correlation_id.clone(),
        root_id: request.allowlisted_root.id.clone(),
        artifact_id: request.artifact_id.clone(),
        operational_report_id: report.operational_report_id,
        headquarters_id: report.headquarters_id,
        department_id: report.department_id,
        reporting_team_id: report.reporting_team_id,
        schema_id: DEPARTMENT_DAILY_REPORT_SCHEMA,
        media_type: REPORT_MEDIA_TYPE,
        size_bytes,
        digest_sha256,
        classification: classification_name(report.classification).to_owned(),
        customer_data: report.customer_data,
        external_actions: false,
    }
}

fn receipt_id(request: &MountedSharePointReportRequest, digest_sha256: &str) -> String {
    let mut digest = Sha256::new();
    for value in [
        request.request_id.as_str(),
        request.correlation_id.as_str(),
        request.artifact_id.as_str(),
        request.allowlisted_root.id.as_str(),
        digest_sha256,
    ] {
        digest.update((value.len() as u64).to_be_bytes());
        digest.update(value.as_bytes());
    }
    format!("receipt-{}", &lower_hex(&digest.finalize())[..24])
}

pub(super) fn lower_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("writing to a string cannot fail");
    }
    encoded
}

const fn classification_name(classification: ReportClassification) -> &'static str {
    match classification {
        ReportClassification::Internal => "internal",
        ReportClassification::InternalConfidential => "internal-confidential",
        ReportClassification::PersonalConfidential => "personal-confidential",
        ReportClassification::RestrictedSensitive => "restricted-sensitive",
        ReportClassification::CustomerConfidential => "customer-confidential",
    }
}
