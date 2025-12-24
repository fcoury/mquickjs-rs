use mquickjs_rs::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::with_memory(512 * 1024)?;
    let ctx = runtime.context()?;

    let result = ctx.eval_i32("1 + 2 + 3", "runtime")?;
    println!("sum: {result}");

    Ok(())
}
