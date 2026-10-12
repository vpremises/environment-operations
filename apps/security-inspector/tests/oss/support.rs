//! Isolated repositories and bounded archive fixtures contain no real credentials.
use serde_json::json;
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

pub fn write(root: &Path, name: &str, contents: impl AsRef<[u8]>) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}
pub fn git(root: &Path, args: &[&str]) {
    assert!(Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .unwrap()
        .success());
}
pub fn repository() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "--quiet"]);
    for name in [
        "LICENSE",
        "NOTICE",
        "CONTRIBUTING.md",
        "CODE_OF_CONDUCT.md",
        "SECURITY.md",
        ".github/pull_request_template.md",
        ".github/ISSUE_TEMPLATE/config.yml",
    ] {
        write(root.path(), name, "Public document\n");
    }
    write(
        root.path(),
        ".github/oss-policy.json",
        json!({"version":1,"license":"Apache-2.0",
        "allowed_vendor_archives":[],"allowed_source_files":[]})
        .to_string(),
    );
    git(root.path(), &["add", "."]);
    root
}
pub fn manifest() -> serde_json::Value {
    json!({"name":"@example/library","version":"0.1.0","license":"Apache-2.0",
    "repository":"git+https://github.com/example/library.git", "files":["dist"],
    "publishConfig":{"registry":"https://registry.npmjs.org"},
    "exports":{".":{"types":"./dist/index.d.ts","import":"./dist/index.js"}}})
}
pub fn package() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "package.json", manifest().to_string());
    for name in ["LICENSE", "NOTICE", "README.md"] {
        write(root.path(), name, "Public document\n");
    }
    root
}
pub fn archive(root: &Path, entries: &[(&str, &[u8])]) -> std::path::PathBuf {
    let output = root.join("candidate.tgz");
    let encoder = flate2::write::GzEncoder::new(
        fs::File::create(&output).unwrap(),
        flate2::Compression::fast(),
    );
    let mut builder = tar::Builder::new(encoder);
    for (name, bytes) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len().try_into().unwrap());
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, name, *bytes).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap();
    output
}
pub fn has(report: &vpremises::oss::OssReport, code: &str) -> bool {
    report
        .findings
        .iter()
        .chain(&report.uninspected)
        .any(|finding| finding.code == code)
}
