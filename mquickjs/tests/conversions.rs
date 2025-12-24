use mquickjs_rs::{Context, FromValue, IntoValue, JsError};

#[test]
fn from_value_converts_primitives() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let value = ctx.eval("true", "test").expect("eval should succeed");
    let result = <bool as FromValue>::from_value(value).expect("bool should convert");
    assert!(result);

    let value = ctx.eval("42", "test").expect("eval should succeed");
    let result = <i32 as FromValue>::from_value(value).expect("i32 should convert");
    assert_eq!(result, 42);

    let value = ctx.eval("3.5", "test").expect("eval should succeed");
    let result = <f64 as FromValue>::from_value(value).expect("f64 should convert");
    assert!((result - 3.5).abs() < f64::EPSILON);

    let value = ctx.eval("'hello'", "test").expect("eval should succeed");
    let result = <String as FromValue>::from_value(value).expect("string should convert");
    assert_eq!(result, "hello");
}

#[test]
fn from_value_rejects_wrong_type() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval("'hello'", "test").expect("eval should succeed");
    let err = <i32 as FromValue>::from_value(value).expect_err("expected conversion error");
    assert!(matches!(err, JsError::Conversion { .. }));
}

#[test]
fn into_value_roundtrips_primitives() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let value = true.into_value(&ctx).expect("bool should convert");
    assert_eq!(value.to_bool().expect("bool should read"), true);

    let value = 7i32.into_value(&ctx).expect("i32 should convert");
    assert_eq!(value.to_i32().expect("i32 should read"), 7);

    let value = 2.25f64.into_value(&ctx).expect("f64 should convert");
    let result = value.to_f64().expect("f64 should read");
    assert!((result - 2.25).abs() < f64::EPSILON);

    let value = "hello".into_value(&ctx).expect("&str should convert");
    assert_eq!(value.to_string().expect("string should read"), "hello");

    let value = String::from("world")
        .into_value(&ctx)
        .expect("String should convert");
    assert_eq!(value.to_string().expect("string should read"), "world");
}

#[test]
fn vec_conversions_roundtrip() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let value = ctx.eval("[1, 2, 3]", "test").expect("eval should succeed");
    let result = <Vec<i32> as FromValue>::from_value(value).expect("vec should convert");
    assert_eq!(result, vec![1, 2, 3]);

    let input = vec![4, 5, 6];
    let value = input
        .clone()
        .into_value(&ctx)
        .expect("vec should convert");
    let result = <Vec<i32> as FromValue>::from_value(value).expect("vec should convert");
    assert_eq!(result, input);
}
