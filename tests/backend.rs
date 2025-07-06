use rustdbgen::infer_backend_from_url;

#[test]
fn infer_backend_from_url_parses() {
    assert!(matches!(infer_backend_from_url("postgres://localhost"), Some(rustdbgen::ir::DatabaseBackend::Postgres)));
    assert!(matches!(infer_backend_from_url("mysql://localhost"), Some(rustdbgen::ir::DatabaseBackend::Mysql)));
    assert!(matches!(infer_backend_from_url("sqlite://test.db"), Some(rustdbgen::ir::DatabaseBackend::Sqlite)));
    assert!(infer_backend_from_url("unknown://foo").is_none());
}
