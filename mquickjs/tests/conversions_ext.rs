use std::collections::{HashMap, VecDeque};

use mquickjs_rs::{Coerced, Context, FromValue, IntoValue};

#[test]
fn option_conversions_handle_null_and_some() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let value = ctx.eval("null", "test").expect("eval should succeed");
    let result = <Option<i32> as FromValue>::from_value(value).expect("option should convert");
    assert!(result.is_none());

    let value = ctx
        .eval("undefined", "test")
        .expect("eval should succeed");
    let result = <Option<String> as FromValue>::from_value(value).expect("option should convert");
    assert!(result.is_none());

    let value = Some(7i32)
        .into_value(&ctx)
        .expect("option should convert");
    let result = <i32 as FromValue>::from_value(value).expect("i32 should convert");
    assert_eq!(result, 7);
}

#[test]
fn numeric_conversions_cover_i64_u64_usize() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let value = 123i64.into_value(&ctx).expect("i64 should convert");
    let result = <i64 as FromValue>::from_value(value).expect("i64 should convert");
    assert_eq!(result, 123);

    let value = 456u64.into_value(&ctx).expect("u64 should convert");
    let result = <u64 as FromValue>::from_value(value).expect("u64 should convert");
    assert_eq!(result, 456);

    let value = 789usize.into_value(&ctx).expect("usize should convert");
    let result = <usize as FromValue>::from_value(value).expect("usize should convert");
    assert_eq!(result, 789);
}

#[test]
fn hash_map_roundtrip() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let mut input = HashMap::new();
    input.insert("a".to_string(), 1i32);
    input.insert("b".to_string(), 2i32);

    let value = input
        .clone()
        .into_value(&ctx)
        .expect("map should convert");
    let output = <HashMap<String, i32> as FromValue>::from_value(value)
        .expect("map should convert");

    assert_eq!(output.len(), input.len());
    assert_eq!(output.get("a"), Some(&1));
    assert_eq!(output.get("b"), Some(&2));
}

#[test]
fn tuple_roundtrip() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let value = (7i32, "hello".to_string())
        .into_value(&ctx)
        .expect("tuple should convert");
    let output = <(i32, String) as FromValue>::from_value(value)
        .expect("tuple should convert");

    assert_eq!(output.0, 7);
    assert_eq!(output.1, "hello");
}

#[test]
fn vecdeque_roundtrip() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let mut input = VecDeque::new();
    input.push_back(1i32);
    input.push_back(2i32);

    let value = input
        .clone()
        .into_value(&ctx)
        .expect("vecdeque should convert");
    let output = <VecDeque<i32> as FromValue>::from_value(value)
        .expect("vecdeque should convert");

    assert_eq!(output.len(), 2);
    assert_eq!(output[0], 1);
    assert_eq!(output[1], 2);
}

#[test]
fn coerced_numeric_and_string() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");

    let value = ctx.eval("'42'", "test").expect("eval should succeed");
    let coerced = <Coerced<i32> as FromValue>::from_value(value)
        .expect("coerced should convert");
    assert_eq!(coerced.0, 42);

    let value = ctx.eval("123", "test").expect("eval should succeed");
    let coerced = <Coerced<String> as FromValue>::from_value(value)
        .expect("coerced should convert");
    assert_eq!(coerced.0, "123");
}
