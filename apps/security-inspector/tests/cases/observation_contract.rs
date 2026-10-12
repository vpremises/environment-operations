//! Configuration contracts bound root cardinality and remain closed.

use crate::support::{observer_config, TempDirectory};
use vpremises::{observe, AllowedRoot, ObserverConfig};

#[test]
fn observer_config_rejects_unknown_fields() {
    let example = include_str!("../../examples/observer.repository.json");
    serde_json::from_str::<ObserverConfig>(example).expect("committed example is valid");
    let source = example.replacen("\n}", ",\n  \"unexpected\": true\n}", 1);
    assert!(serde_json::from_str::<ObserverConfig>(&source).is_err());
}

#[test]
fn observer_rejects_more_than_256_explicit_roots() {
    let root = TempDirectory::create();
    let mut config = observer_config(&root.0);
    config.roots = (0..257)
        .map(|index| AllowedRoot {
            id: format!("root-{index}"),
            path: root.0.clone(),
        })
        .collect();
    let report = observe(&config);
    assert!(report
        .diagnostics
        .iter()
        .any(|item| item.code == "vpremises.roots.too-many"));
}

#[test]
fn bundled_observer_schema_matches_collection_limit() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../schemas/vpremises.observer.v1.schema.json"
    ))
    .expect("schema JSON");
    assert_eq!(schema["properties"]["roots"]["maxItems"], 256);
}
