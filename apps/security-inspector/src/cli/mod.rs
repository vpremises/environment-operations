//! CLI routing preserves JSON-only output and delegates observation policy to the library.
mod audit;
mod doctor;
mod file_input;
mod oss;
mod output;
mod report_request;
mod workspace;

#[cfg(test)]
mod tests;

use serde_json::{json, Value};
use std::{env, path::Path, process::ExitCode};
use vpremises::{acquire_mounted_sharepoint_report, observe, security_report};

pub(super) fn main() -> ExitCode {
    let Some(mut arguments) = env::args_os()
        .skip(1)
        .map(|value| value.into_string().ok())
        .collect::<Option<Vec<_>>>()
    else {
        output::emit(&output::usage_error(), false);
        return ExitCode::from(1);
    };
    let jsonl = arguments
        .last()
        .is_some_and(|argument| argument == "--jsonl");
    if jsonl {
        arguments.pop();
    }
    match run(&arguments) {
        Ok(value) => {
            output::emit(&value, jsonl);
            ExitCode::SUCCESS
        }
        Err((code, value)) => {
            output::emit(&value, jsonl);
            ExitCode::from(code)
        }
    }
}

fn run(arguments: &[String]) -> Result<Value, (u8, Value)> {
    match arguments {
        [command, rest @ ..] if command == "oss" => oss::run(rest),
        [command] if command == "doctor" => Ok(doctor::doctor()),
        [command, directory, root] if command == "init" => {
            audit::initialize(Path::new(directory), Path::new(root))
        }
        [command, path, environment] if command == "audit" => {
            audit::run(Path::new(path), environment)
        }
        [command, path, environment] if command == "report" => {
            let config = workspace::load_observer_config(Path::new(path))?;
            let report = security_report(&config, environment).map_err(|message| {
                (
                    1,
                    json!({"ok": false, "code": "report-invalid", "message": message}),
                )
            })?;
            let value = serde_json::to_value(report).expect("security report serializes");
            Err((2, value))
        }
        [command, path] if command == "observe" => {
            let report = observe(&workspace::load_observer_config(Path::new(path))?);
            let ok = report.ok;
            let value = serde_json::to_value(report).expect("observation report serializes");
            if ok {
                Ok(value)
            } else {
                Err((2, value))
            }
        }
        [command, path] if command == "acquire-mounted-sharepoint-report" => {
            acquire_report(Path::new(path))
        }
        _ => Err((1, output::usage_error())),
    }
}

fn acquire_report(path: &Path) -> Result<Value, (u8, Value)> {
    let request = report_request::load(path)?;
    let root_base = report_request::resolve_root_base(path, request.allowlisted_root.base)?;
    acquire_mounted_sharepoint_report(&request, &root_base)
        .map(|receipt| serde_json::to_value(receipt).expect("receipt serializes"))
        .map_err(|error| {
            (
                2,
                json!({
                    "schema": "vpremises://errors/mounted-sharepoint-report/v1",
                    "ok": false,
                    "code": error.code,
                    "field": error.field,
                    "message": error.message,
                    "external_actions": false
                }),
            )
        })
}
