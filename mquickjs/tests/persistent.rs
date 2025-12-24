use mquickjs_rs::{Context, Persistent};

#[test]
fn persistent_survives_gc_and_clone() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval("'hello'", "test").expect("eval should succeed");
    let persistent = Persistent::new(&ctx, value).expect("persist should succeed");
    let clone = persistent.clone();

    drop(persistent);
    for _ in 0..3 {
        ctx.gc();
    }

    let roundtrip = clone.to_value().to_string().expect("to_string");
    assert_eq!(roundtrip, "hello");
}

#[test]
fn persistent_drop_is_safe() {
    let ctx = Context::new(1024 * 1024).expect("context should initialize");
    let value = ctx.eval("({ value: 42 })", "test").expect("eval should succeed");
    let persistent = Persistent::new(&ctx, value).expect("persist should succeed");
    drop(persistent);
    ctx.gc();
}
