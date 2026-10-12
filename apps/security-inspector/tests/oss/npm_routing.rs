//! Source and packed manifests share explicit registry routing gates.
use super::support::{has, manifest, package, write};
use vpremises::oss::inspect_package;

#[test]
fn registry_routing_is_identical_for_source_and_packed_manifests() {
    use super::support::archive;
    for (selector, expected) in [
        ("owner/repository", true),
        ("github:owner/repository", true),
        ("git+ssh://host/repo", true),
        ("https://host/pkg.tgz", true),
        ("npm:example@github:owner/repo", true),
        ("^1.2", false),
        ("npm:@scope/example@latest", false),
    ] {
        let root = package();
        let mut value = manifest();
        value["dependencies"] = serde_json::json!({"example": selector});
        let text = value.to_string();
        write(root.path(), "package.json", &text);
        let source = inspect_package(root.path(), "npm", None).unwrap();
        let packed = archive(root.path(), &[("package/package.json", text.as_bytes())]);
        let packed = inspect_package(root.path(), "npm", Some(&packed)).unwrap();
        assert_eq!(
            has(&source, "npm-nonregistry-dependency"),
            expected,
            "source: {selector}"
        );
        assert_eq!(
            has(&packed, "npm-nonregistry-dependency"),
            expected,
            "packed: {selector}"
        );
    }
}
