//! CLI path-resolution tests prove configuration packages remain portable.

use super::workspace::load_observer_config;
use std::{fs, path::PathBuf};

#[test]
fn observer_root_resolves_from_its_configuration_package() {
    let package = temporary_directory("observer-root");
    let observed = package.join("observed");
    fs::create_dir(&observed).expect("observed directory");
    let config_path = package.join("observer.json");
    fs::write(
        &config_path,
        r#"{
          "schema_version":"vpremises.observer/v1",
          "roots":[{"id":"package-data","path":"observed"}],
          "limits":{"max_depth":4,"max_entries":100,"max_total_bytes":10000},
          "policy":{"metadata_only":true,"follow_symlinks":false}
        }"#,
    )
    .expect("observer configuration");
    let config = load_observer_config(&config_path).expect("portable observer configuration");
    assert_eq!(config.roots.len(), 1);
    assert_eq!(
        config.roots[0].path,
        observed.canonicalize().expect("canonical observed root")
    );
    assert!(config.roots[0].path.is_absolute());
    fs::remove_dir_all(package).expect("cleanup");
}

#[test]
fn dot_observer_root_selects_only_the_configuration_package() {
    let package = temporary_directory("dot-root");
    let config_path = package.join("observer.json");
    fs::write(
        &config_path,
        r#"{
          "schema_version":"vpremises.observer/v1",
          "roots":[{"id":"package","path":"."}],
          "limits":{"max_depth":4,"max_entries":100,"max_total_bytes":10000},
          "policy":{"metadata_only":true,"follow_symlinks":false}
        }"#,
    )
    .expect("observer configuration");
    let config = load_observer_config(&config_path).expect("package observer configuration");
    assert_eq!(
        config.roots[0].path,
        package.canonicalize().expect("canonical package root")
    );
    fs::remove_dir_all(package).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn bounded_cli_reader_rejects_a_symlinked_document() {
    use std::{fs, os::unix::fs::symlink};

    let directory = std::env::temp_dir().join(format!("vpremises-cli-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).expect("test directory");
    let target = directory.join("target.json");
    let linked = directory.join("linked.json");
    fs::write(&target, "{}").expect("target");
    symlink(&target, &linked).expect("symlink");
    assert!(super::file_input::read_utf8(&linked, 64).is_err());
    fs::remove_dir_all(directory).expect("cleanup");
}

fn temporary_directory(label: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "vpremises-{label}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir(&directory).expect("temporary directory");
    directory
}
