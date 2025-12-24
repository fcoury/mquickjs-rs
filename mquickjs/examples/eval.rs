use mquickjs_rs::{Context, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new(1024 * 1024)?;

    let sum = ctx.eval_i32("1 + 2 + 3", "eval")?;
    println!("sum: {sum}");

    ctx.register_fn("echo", |args: &[Value<'_>]| Ok(args[0]))?;
    let echoed = ctx.eval_string("echo('hello')", "eval")?;
    println!("echoed: {echoed}");

    Ok(())
}
