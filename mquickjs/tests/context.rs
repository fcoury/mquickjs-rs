use mquickjs::{Context, JsError};

#[test]
fn context_requires_minimum_memory() {
    let err = Context::new(512).expect_err("expected error for too-small buffer");
    assert!(matches!(err, JsError::ContextInit { .. }));
}

#[test]
fn context_new_creates_and_drops() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    drop(ctx);
}
