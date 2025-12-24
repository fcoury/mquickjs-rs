use mquickjs_rs::Runtime;

#[test]
fn runtime_creates_context_and_eval() {
    let runtime = Runtime::new().expect("runtime should initialize");
    let ctx = runtime.context().expect("context should initialize");
    let result = ctx.eval_i32("1 + 2", "test").expect("eval should succeed");
    assert_eq!(result, 3);
}

#[test]
fn runtime_builder_configures_memory() {
    let runtime = Runtime::with_memory(512 * 1024).expect("runtime should initialize");
    let ctx = runtime.context().expect("context should initialize");
    let result = ctx.eval_i32("4 + 4", "test").expect("eval should succeed");
    assert_eq!(result, 8);
}
