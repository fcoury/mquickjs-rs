use mquickjs_rs::Context;

#[test]
fn rooted_value_survives_gc() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval("'hello'", "test").expect("eval should succeed");
    let rooted = ctx.root(value);

    for _ in 0..5 {
        ctx.eval("var a = []; for (var i = 0; i < 1000; i++) { a[i] = i; } a;", "alloc")
            .expect("alloc");
        ctx.gc();
    }

    let roundtrip = rooted.to_value().to_string().expect("to_string");
    assert_eq!(roundtrip, "hello");
}

#[test]
fn dropping_rooted_value_does_not_crash() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval("'temp'", "test").expect("eval should succeed");
    let rooted = ctx.root(value);

    drop(rooted);
    ctx.gc();

    let value = ctx.eval_i32("1 + 1", "test").expect("eval should succeed");
    assert_eq!(value, 2);
}
