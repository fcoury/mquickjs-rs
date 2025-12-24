use mquickjs_rs::{Context, JsError, Value};

#[test]
fn register_fn_invokes_callback() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    ctx.register_fn("echo", |args: &[Value<'_>]| {
        assert_eq!(args.len(), 1);
        let value = args[0].to_i32()?;
        assert_eq!(value, 3);
        Ok(args[0])
    })
    .expect("register should succeed");

    let result = ctx.eval_i32("echo(1+2)", "test").expect("eval should succeed");
    assert_eq!(result, 3);
}

#[test]
fn register_fn_propagates_error() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    ctx.register_fn("fail", |_args: &[Value<'_>]| {
        Err(JsError::Callback {
            message: "boom".to_string(),
        })
    })
    .expect("register should succeed");

    let err = ctx
        .eval_i32("fail(1)", "test")
        .expect_err("expected runtime error");
    assert!(matches!(err, JsError::Exception { .. }));
}
