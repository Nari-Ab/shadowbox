#[test]
fn test_version() {
    assert_eq!(shadowbox::version(), env!("CARGO_PKG_VERSION"));
}
