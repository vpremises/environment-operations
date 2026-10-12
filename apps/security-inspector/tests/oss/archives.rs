//! Distribution tests never extract unsafe members to the filesystem.
use super::support::{archive, has, manifest, package, write};
use vpremises::oss::inspect_package;

#[test]
fn valid_archive_still_requires_content_detection() {
    let root = package();
    let manifest = manifest().to_string();
    let path = archive(
        root.path(),
        &[
            ("package/package.json", manifest.as_bytes()),
            ("package/LICENSE", b"public"),
            ("package/NOTICE", b"public"),
            ("package/README.md", b"public"),
            ("package/dist/index.js", b"export {};"),
            ("package/dist/index.d.ts", b"export {};"),
        ],
    );
    let report = inspect_package(root.path(), "npm", Some(&path)).unwrap();
    assert!(report.findings.is_empty());
    assert_eq!(report.exit_code(), 2);
    assert!(has(&report, "archive-content-audit-required"));
}
#[test]
fn missing_entry_and_applicable_notice_are_findings() {
    let root = package();
    let manifest = manifest().to_string();
    write(root.path(), "LICENSE-ASSETS", "redistribution notice");
    let path = archive(
        root.path(),
        &[("package/package.json", manifest.as_bytes())],
    );
    let report = inspect_package(root.path(), "npm", Some(&path)).unwrap();
    assert!(has(&report, "npm-entry-missing"));
    assert!(has(&report, "archive-applicable-notice-missing"));
}
#[test]
fn private_paths_duplicates_and_identity_mismatch_fail() {
    let root = package();
    let mut value = manifest();
    value["version"] = "0.2.0".into();
    let text = value.to_string();
    let path = archive(
        root.path(),
        &[
            ("package/package.json", text.as_bytes()),
            ("package/state/private.json", b"synthetic"),
            ("package/state/private.json", b"synthetic"),
        ],
    );
    let report = inspect_package(root.path(), "npm", Some(&path)).unwrap();
    for code in [
        "forbidden-archive-path",
        "duplicate-archive-file",
        "archive-identity-mismatch",
    ] {
        assert!(has(&report, code));
    }
}
#[test]
fn archive_dependency_injection_is_rejected() {
    let root = package();
    let mut value = manifest();
    value["dependencies"] = serde_json::json!({"hidden":"file:../hidden"});
    let text = value.to_string();
    let path = archive(root.path(), &[("package/package.json", text.as_bytes())]);
    assert!(has(
        &inspect_package(root.path(), "npm", Some(&path)).unwrap(),
        "npm-nonregistry-dependency"
    ));
}
#[test]
fn symlink_and_traversal_headers_never_get_extracted() {
    let root = package();
    let path = root.path().join("unsafe.tgz");
    let encoder = flate2::write::GzEncoder::new(
        std::fs::File::create(&path).unwrap(),
        flate2::Compression::fast(),
    );
    let mut builder = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    header.set_mode(0o644);
    header.set_link_name("../outside").unwrap();
    builder
        .append_link(&mut header, "package/link", "../outside")
        .unwrap();
    let mut header = tar::Header::new_gnu();
    header.set_size(0);
    header.set_mode(0o644);
    header.as_mut_bytes()[..19].copy_from_slice(b"package/../outside\0");
    header.set_cksum();
    builder.append(&header, std::io::empty()).unwrap();
    builder.into_inner().unwrap().finish().unwrap();
    assert!(has(
        &inspect_package(root.path(), "npm", Some(&path)).unwrap(),
        "unsafe-archive-member"
    ));
    assert!(!root.path().join("outside").exists());
}
#[test]
fn corrupt_crc_and_trailing_stream_are_not_accepted() {
    let root = package();
    let path = archive(root.path(), &[("package/LICENSE", b"test")]);
    let bytes = std::fs::read(&path).unwrap();
    let mut corrupt = bytes.clone();
    let len = corrupt.len();
    corrupt[len - 8] ^= 1;
    std::fs::write(&path, corrupt).unwrap();
    assert!(inspect_package(root.path(), "npm", Some(&path)).is_err());
    let mut trailing = bytes;
    trailing.extend_from_slice(b"hidden trailing stream");
    std::fs::write(&path, trailing).unwrap();
    assert!(inspect_package(root.path(), "npm", Some(&path)).is_err());
}
