use mquickjs_rs::{Array, Context, Object};

#[test]
fn object_set_get_roundtrip() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval("({})", "test").expect("eval should succeed");
    let object = Object::from_value(&ctx, value).expect("object should wrap");

    object.set("answer", 42i32).expect("set should succeed");
    object
        .set("title", "mquickjs")
        .expect("set should succeed");

    let answer: i32 = object.get("answer").expect("get should succeed");
    assert_eq!(answer, 42);

    let title: String = object.get("title").expect("get should succeed");
    assert_eq!(title, "mquickjs");
}

#[test]
fn array_push_get_roundtrip() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval("[]", "test").expect("eval should succeed");
    let array = Array::from_value(&ctx, value).expect("array should wrap");

    array.push(1i32).expect("push should succeed");
    array.push(2i32).expect("push should succeed");

    let first: i32 = array.get(0).expect("get should succeed");
    let second: i32 = array.get(1).expect("get should succeed");
    assert_eq!(first, 1);
    assert_eq!(second, 2);
}
