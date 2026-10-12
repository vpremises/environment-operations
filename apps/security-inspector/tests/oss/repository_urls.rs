//! Accept supported npm HTTPS metadata while rejecting misleading repository paths.
use super::support::{has, manifest, package, write};
use vpremises::oss::inspect_package;

#[test]
fn npm_https_repository_variants_pass() {
    for url in [
        "https://github.com/example/library.git",
        "git+https://github.com/example/library.git",
    ] {
        let root = package();
        let mut value = manifest();
        value["repository"] = url.into();
        write(root.path(), "package.json", value.to_string());
        assert_eq!(
            inspect_package(root.path(), "npm", None)
                .unwrap()
                .exit_code(),
            0
        );
    }
}

#[test]
fn misleading_repository_urls_are_rejected() {
    for url in [
        "git+https://github.com/../library.git",
        "git+https://github.com/example/..git",
        "git+https://github.com/example/library.git?token=invalid-fixture",
        "git+https://user:invalid-fixture@github.com/example/library.git",
        "git+https://github.com.evil.invalid/example/library.git",
    ] {
        let root = package();
        let mut value = manifest();
        value["repository"] = url.into();
        write(root.path(), "package.json", value.to_string());
        assert!(has(
            &inspect_package(root.path(), "npm", None).unwrap(),
            "package-repository-invalid"
        ));
    }
}
