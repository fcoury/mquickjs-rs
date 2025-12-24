use mquickjs::{Context, JsError};

#[test]
fn context_new_is_stubbed() {
    let err = Context::new(1024).expect_err("expected stub to error");
    assert!(matches!(err, JsError::Unimplemented));
}
