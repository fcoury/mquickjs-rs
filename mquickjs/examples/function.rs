use mquickjs_rs::{Function, IntoValue, Runtime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Runtime::new()?;
    let ctx = runtime.context()?;

    let value = ctx.eval("(function(a, b) { return a * b; })", "example")?;
    let func = Function::from_value(&ctx, value)?;

    let arg1 = 3i32.into_value(&ctx)?;
    let arg2 = 4i32.into_value(&ctx)?;
    let result = func.call(&[arg1, arg2])?;
    let value = result.to_i32()?;
    println!("product: {value}");

    Ok(())
}
