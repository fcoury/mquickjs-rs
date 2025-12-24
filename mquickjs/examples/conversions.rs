use mquickjs_rs::{Array, Context, FromValue, IntoValue, Object};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Context::new(1024 * 1024)?;

    let value = 42i32.into_value(&ctx)?;
    let result = i32::from_value(value)?;
    println!("roundtrip: {result}");

    let obj_value = ctx.eval("({})", "example")?;
    let obj = Object::from_value(&ctx, obj_value)?;
    obj.set("name", "mquickjs")?;
    let name: String = obj.get("name")?;
    println!("name: {name}");

    let array_value = ctx.eval("[]", "example")?;
    let array = Array::from_value(&ctx, array_value)?;
    array.push(1i32)?;
    array.push(2i32)?;
    let first: i32 = array.get(0)?;
    let second: i32 = array.get(1)?;
    println!("array: {first}, {second}");

    Ok(())
}
