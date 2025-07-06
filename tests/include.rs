use rustdbgen::load_schema;

#[test]
fn includes_merge_models() {
    let ir = load_schema("schema.model.toml").unwrap();
    assert!(ir.models.contains_key("User"));
    assert!(ir.models.contains_key("Post"));
}

#[test]
fn duplicate_model_names_error() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let inc = dir.path().join("inc.toml");
    fs::write(
        &inc,
        "[models.Dupe]\nfields.id.type='Uuid'\nfields.id.db_type='UUID'",
    )
    .unwrap();
    let main = dir.path().join("main.toml");
    fs::write(
        &main,
        format!(
            "include = ['{}']\n\n[models.Dupe]\nfields.id.type='Uuid'\nfields.id.db_type='UUID'",
            inc.to_string_lossy()
        ),
    )
    .unwrap();
    let res = load_schema(main.to_str().unwrap());
    assert!(res.is_err(), "expected duplicate model error");
}

#[test]
fn duplicate_enum_names_error() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let inc = dir.path().join("inc.toml");
    fs::write(&inc, "[enums.Thing]\nvariants=['A']").unwrap();
    let main = dir.path().join("main.toml");
    fs::write(
        &main,
        format!(
            "include = ['{}']\n\n[enums.Thing]\nvariants=['B']",
            inc.to_string_lossy()
        ),
    )
    .unwrap();
    let res = load_schema(main.to_str().unwrap());
    assert!(res.is_err(), "expected duplicate enum error");
}

#[test]
fn include_cycle_errors() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let a = dir.path().join("a.toml");
    let b = dir.path().join("b.toml");
    fs::write(&a, format!("include = ['{}']", b.to_string_lossy())).unwrap();
    fs::write(&b, format!("include = ['{}']", a.to_string_lossy())).unwrap();
    let res = load_schema(a.to_str().unwrap());
    assert!(res.is_err(), "expected cyclic include error");
}
