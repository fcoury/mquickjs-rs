use mquickjs_rs::{Context, JsError};

#[test]
fn eval_i32_returns_value() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval_i32("1+2", "test").expect("eval should succeed");
    assert_eq!(value, 3);
}

#[test]
fn eval_bool_returns_value() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval_bool("1 < 2", "test").expect("eval should succeed");
    assert!(value);
}

#[test]
fn eval_string_returns_value() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx
        .eval_string("'hello'", "test")
        .expect("eval should succeed");
    assert_eq!(value, "hello");
}

#[test]
fn eval_invalid_js_returns_error() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let err = ctx
        .eval_i32("let =", "bad")
        .expect_err("expected eval error");
    assert!(matches!(err, JsError::Exception { .. }));
}
