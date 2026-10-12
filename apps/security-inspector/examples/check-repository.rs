//! Use the same OSS implementation as local operators and CI.
use std::{path::Path, process::ExitCode};

fn main() -> ExitCode {
    match vpremises::oss::inspect_repository(Path::new("."), None) {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("JSON receipt")
            );
            ExitCode::from(report.exit_code())
        }
        Err(code) => {
            eprintln!("OSS inspection incomplete: {code}");
            ExitCode::from(2)
        }
    }
}
