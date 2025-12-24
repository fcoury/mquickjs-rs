use mquickjs_rs::{Context, Function, IntoValue};

#[test]
fn function_call_with_args() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx
        .eval("(function(a, b) { return a + b; })", "test")
        .expect("eval should succeed");
    let func = Function::from_value(&ctx, value).expect("function should wrap");

    let arg1 = 2i32.into_value(&ctx).expect("arg should convert");
    let arg2 = 3i32.into_value(&ctx).expect("arg should convert");

    let result = func.call(&[arg1, arg2]).expect("call should succeed");
    let sum = result.to_i32().expect("result should convert");
    assert_eq!(sum, 5);
}

#[test]
fn function_call0() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx
        .eval("(function() { return 7; })", "test")
        .expect("eval should succeed");
    let func = Function::from_value(&ctx, value).expect("function should wrap");

    let result = func.call0().expect("call should succeed");
    let value = result.to_i32().expect("result should convert");
    assert_eq!(value, 7);
}
