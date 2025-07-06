use rustdbgen::ir::SchemaIR;
use rustdbgen::{load_schema, run_plugin};
use std::collections::HashMap;

#[test]
fn plugin_cat_outputs_json() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let out = run_plugin("cat", &[], &HashMap::new(), &None, &ir).unwrap();
    let parsed: SchemaIR = serde_json::from_str(&out).unwrap();
    assert_eq!(parsed.schema_version, ir.schema_version);
    assert_eq!(parsed.models.len(), ir.models.len());
}

#[test]
fn named_plugin_from_schema() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let def = ir.plugins.get("cat").unwrap();
    let out = run_plugin(&def.command, &def.args, &def.env, &def.cwd, &ir).unwrap();
    let parsed: SchemaIR = serde_json::from_str(&out).unwrap();
    assert_eq!(parsed.models.len(), ir.models.len());
}

#[test]
fn plugin_with_default_args() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let def = ir.plugins.get("echo").unwrap();
    let out = run_plugin(&def.command, &def.args, &def.env, &def.cwd, &ir).unwrap();
    assert_eq!(out.trim(), "hello");
}

#[test]
fn plugin_with_extra_args() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let out = run_plugin(
        "sh",
        &["-c".into(), "cat >/dev/null; echo ok".into()],
        &HashMap::new(),
        &None,
        &ir,
    )
    .unwrap();
    assert_eq!(out.trim(), "ok");
}

#[test]
fn plugin_with_env_vars() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let def = ir.plugins.get("envtest").unwrap();
    let out = run_plugin(&def.command, &def.args, &def.env, &def.cwd, &ir).unwrap();
    assert_eq!(out.trim(), "bar");
}

#[test]
fn plugin_with_cwd() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let def = ir.plugins.get("pwdtest").unwrap();
    let out = run_plugin(&def.command, &def.args, &def.env, &def.cwd, &ir).unwrap();
    assert!(out.trim().ends_with("tests"));
}

#[test]
fn duplicate_plugin_names_error() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let inc = dir.path().join("inc.toml");
    fs::write(&inc, "[plugins.foo]\ncommand='echo'").unwrap();
    let main = dir.path().join("main.toml");
    fs::write(
        &main,
        format!(
            "include = ['{}']\n\n[plugins.foo]\ncommand='echo'",
            inc.to_string_lossy()
        ),
    )
    .unwrap();
    let res = load_schema(main.to_str().unwrap());
    assert!(res.is_err(), "expected duplicate plugin error");
}
