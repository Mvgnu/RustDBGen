use rustdbgen::{pull_schema, push_schema};
use std::fs;
use tempfile::tempdir;

#[test]
fn registry_push_pull_round_trip() {
    let dir = tempdir().unwrap();
    let remote = dir.path().join("schema.toml");
    push_schema("schema.model.toml", remote.to_str().unwrap()).unwrap();
    let local = dir.path().join("copy.toml");
    pull_schema(remote.to_str().unwrap(), local.to_str().unwrap()).unwrap();
    let orig = fs::read_to_string("schema.model.toml").unwrap();
    let copied = fs::read_to_string(local).unwrap();
    assert_eq!(orig, copied);
}
