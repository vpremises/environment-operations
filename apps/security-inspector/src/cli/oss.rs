//! OSS subcommands preserve a common JSON receipt and fail-closed exit semantics.
use serde_json::{json, Value};
use std::path::Path;
use vpremises::oss::{inspect_exclusions, inspect_package, inspect_repository, OssReport};

pub(super) fn run(args: &[String]) -> Result<Value, (u8, Value)> {
    if let [command, gate, root] = args {
        if command == "workspace" {
            return match vpremises::oss::inspect_workspace(Path::new(root), gate) {
                Ok(report) => {
                    let code = report.exit_code();
                    let value = serde_json::to_value(report).expect("workspace receipt");
                    if code == 0 {
                        Ok(value)
                    } else {
                        Err((code, value))
                    }
                }
                Err(code) => Err((
                    2,
                    json!({"schema":"vpremises-security/oss-error/v1", "status":"incomplete", "code":code, "external_actions":false}),
                )),
            };
        }
    }
    match inspect(args) {
        Ok(report) => {
            let code = report.exit_code();
            let value = serde_json::to_value(report).expect("OSS receipt serializes");
            if code == 0 {
                Ok(value)
            } else {
                Err((code, value))
            }
        }
        Err(code) => Err((
            2,
            json!({"schema":"vpremises-security/oss-error/v1",
            "status":"incomplete", "code":code, "external_actions":false}),
        )),
    }
}

fn inspect(args: &[String]) -> Result<OssReport, &'static str> {
    match args {
        [command, root, rest @ ..] if ["repository", "exclusions"].contains(&command.as_str()) => {
            let policy = match rest {
                [] => None,
                [flag, path] if flag == "--policy" => Some(Path::new(path)),
                _ => return Err("oss-usage-invalid"),
            };
            if command == "repository" {
                inspect_repository(Path::new(root), policy)
            } else {
                inspect_exclusions(Path::new(root), policy)
            }
        }
        [command, kind, root, rest @ ..] if command == "package" => {
            let mut archive = None;
            let mut audit = None;
            for pair in rest.chunks(2) {
                match pair {
                    [flag, value] if flag == "--archive" && archive.is_none() => {
                        archive = Some(Path::new(value));
                    }
                    [flag, value] if flag == "--audit" && audit.is_none() => {
                        audit = Some(Path::new(value));
                    }
                    _ => return Err("oss-usage-invalid"),
                }
            }
            let mut report = inspect_package(Path::new(root), kind, archive)?;
            if let Some(config) = audit {
                let archive = archive.ok_or("oss-audit-requires-archive")?;
                vpremises::oss::audit_archive_content(&mut report, archive, config)?;
            }
            Ok(report)
        }
        _ => Err("oss-usage-invalid"),
    }
}
